package aeska.kistaverk

import android.net.Uri
import android.os.Bundle
import android.os.Environment
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.lifecycleScope
import aeska.kistaverk.features.ConversionResult
import aeska.kistaverk.features.KotlinImageConversion
import android.view.View
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import android.content.Intent

class MainActivity : ComponentActivity() {

    private lateinit var renderer: UiRenderer
    private var pendingActionAfterPicker: String? = null
    private var pendingBindingsAfterPicker: Map<String, String> = emptyMap()
    private var selectedOutputDir: Uri? = null
    private var rootContainer: FrameLayout? = null
    private var contentHolder: FrameLayout? = null
    private var overlayView: View? = null

    private val pickFileLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        val action = pendingActionAfterPicker
        val bindings = pendingBindingsAfterPicker
        pendingActionAfterPicker = null
        pendingBindingsAfterPicker = emptyMap()

        if (uri == null || action == null) return@registerForActivityResult

        if (KotlinImageConversion.isConversionAction(action)) {
            handleKotlinImageConversion(uri, action)
            return@registerForActivityResult
        }

        val fd = openFdForUri(uri)
        if (fd != null) {
            dispatchWithOptionalLoading(
                action = action,
                bindings = bindings,
                extras = mapOf("fd" to fd)
            )
        } else {
            // Notify Rust about the failure so it can surface an error state
            dispatchWithOptionalLoading(
                action = action,
                bindings = bindings,
                extras = mapOf<String, Any?>("fd" to JSONObject.NULL, "error" to "open_fd_failed")
            )
        }
    }

    private val pickDirLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocumentTree()
    ) { uri ->
        if (uri != null) {
            try {
                contentResolver.takePersistableUriPermission(
                    uri,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                )
            } catch (_: Exception) {
                // Best-effort; continue even if persist fails
            }
            selectedOutputDir = uri
            refreshUi(
                "kotlin_image_output_dir",
                mapOf(
                    "output_dir" to uri.toString()
                )
            )
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        renderer = UiRenderer(this) { action, needsFilePicker, bindings ->
            if (action == "kotlin_image_pick_dir") {
                pickDirLauncher.launch(null)
                return@UiRenderer
            }
            if (action == "progress_demo_start") {
                lifecycleScope.launch {
                    showOverlay("Simulating work...")
                    refreshUi(action, bindings = bindings, loadingOnly = true)
                    kotlinx.coroutines.delay(10_000)
                    refreshUi("progress_demo_finish")
                }
                return@UiRenderer
            }

            if (needsFilePicker) {
                pendingActionAfterPicker = action
                pendingBindingsAfterPicker = bindings
                pickFileLauncher.launch(arrayOf("*/*"))
            } else {
                if (action == "reset") {
                    selectedOutputDir = null
                }
                dispatchWithOptionalLoading(action, bindings = bindings)
            }
        }

        // Initial Load
        refreshUi("init")
    }

    private fun refreshUi(
        action: String,
        extras: Map<String, Any?> = emptyMap(),
        bindings: Map<String, String> = emptyMap(),
        loadingOnly: Boolean = false
    ) {
        lifecycleScope.launch {
            val command = JSONObject().apply {
                put("action", action)
                extras.forEach { (k, v) ->
                    // JSONObject handles proper escaping; null maps to JSON null
                    put(k, v)
                }
                if (bindings.isNotEmpty()) {
                    val bindingsObj = JSONObject()
                    bindings.forEach { (k, v) -> bindingsObj.put(k, v) }
                    put("bindings", bindingsObj)
                }
                if (loadingOnly) {
                    put("loading_only", true)
                }
            }

            val newUiJson = withContext(Dispatchers.IO) {
                dispatch(command.toString())
            }

            if (loadingOnly) {
                showOverlay(command.optString("action", "Working..."))
                return@launch
            }

            val rootView = runCatching { renderer.render(newUiJson) }
                .getOrElse { throwable ->
                    renderer.renderFallback(
                        title = "Render error",
                        message = throwable.message ?: "unknown_render_error"
                    )
                }
            attachContent(rootView)
            hideOverlay()
        }
    }

    private fun openFdForUri(uri: Uri): Int? {
        return try {
            contentResolver.openFileDescriptor(uri, "r")?.use { pfd ->
                pfd.detachFd().takeIf { it >= 0 }
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun handleKotlinImageConversion(uri: Uri, action: String) {
        lifecycleScope.launch {
            val result = withContext(Dispatchers.IO) {
                KotlinImageConversion.convert(
                    context = this@MainActivity,
                    cacheDir = cacheDir,
                    picturesDir = getExternalFilesDir(Environment.DIRECTORY_PICTURES),
                    outputDirUri = selectedOutputDir,
                    uri = uri,
                    action = action
                )
            }

            when (result) {
                is ConversionResult.Success -> refreshUi(
                    "kotlin_image_result",
                    mapOf(
                        "target" to result.target.key,
                        "result_path" to result.destination,
                        "result_size" to result.size,
                        "result_format" to result.format
                    )
                )
                is ConversionResult.Failure -> {
                    val reason = result.reason ?: "conversion_failed"
                    refreshUi(
                        "kotlin_image_result",
                        mapOf(
                            "target" to (result.target?.key ?: JSONObject.NULL),
                            "error" to reason
                        )
                    )
                }
            }
        }
    }

    private fun dispatchWithOptionalLoading(
        action: String,
        bindings: Map<String, String> = emptyMap(),
        extras: Map<String, Any?> = emptyMap()
    ) {
        val isHashAction = action.startsWith("hash_file_")
        if (isHashAction) {
            showOverlay("Computing hash...")
            refreshUi(action, bindings = bindings, loadingOnly = true)
        }
        refreshUi(action, extras = extras, bindings = bindings)
    }

    private fun ensureContainers() {
        if (rootContainer != null && contentHolder != null && overlayView != null) return

        rootContainer = FrameLayout(this).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        }
        contentHolder = FrameLayout(this).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        }
        overlayView = buildOverlay()
        rootContainer!!.addView(contentHolder)
        rootContainer!!.addView(overlayView)
        setContentView(rootContainer)
    }

    private fun attachContent(view: View) {
        ensureContainers()
        contentHolder?.removeAllViews()
        contentHolder?.addView(view)
    }

    private fun buildOverlay(): View {
        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            setBackgroundColor(0x88000000.toInt())
            visibility = View.GONE
            val padding = (resources.displayMetrics.density * 24).toInt()
            setPadding(padding, padding, padding, padding)
        }
        val text = TextView(this).apply {
            text = "Working..."
            textSize = 16f
            setTextColor(0xFFFFFFFF.toInt())
        }
        val bar = ProgressBar(this).apply {
            isIndeterminate = true
        }
        container.addView(text)
        container.addView(bar)
        return container
    }

    private fun showOverlay(message: String) {
        ensureContainers()
        val textView = (overlayView as? LinearLayout)?.getChildAt(0) as? TextView
        textView?.text = message
        overlayView?.visibility = View.VISIBLE
    }

    private fun hideOverlay() {
        overlayView?.visibility = View.GONE
    }

    external fun dispatch(input: String): String

    companion object {
        init {
            System.loadLibrary("kistaverk_core")
        }
    }
}
