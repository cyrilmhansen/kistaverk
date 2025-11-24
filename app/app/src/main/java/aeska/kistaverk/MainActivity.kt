package aeska.kistaverk

import android.net.Uri
import android.os.Bundle
import android.os.Environment
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.lifecycleScope
import aeska.kistaverk.features.ConversionResult
import aeska.kistaverk.features.KotlinImageConversion
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import android.content.Intent

class MainActivity : ComponentActivity() {

    private lateinit var renderer: UiRenderer
    private var pendingActionAfterPicker: String? = null
    private var selectedOutputDir: Uri? = null

    private val pickFileLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        val action = pendingActionAfterPicker
        pendingActionAfterPicker = null

        if (uri == null || action == null) return@registerForActivityResult

        if (KotlinImageConversion.isConversionAction(action)) {
            handleKotlinImageConversion(uri, action)
            return@registerForActivityResult
        }

        val fd = openFdForUri(uri)
        if (fd != null) {
            refreshUi(action, mapOf("fd" to fd))
        } else {
            // Notify Rust about the failure so it can surface an error state
            refreshUi(action, mapOf<String, Any?>("fd" to JSONObject.NULL, "error" to "open_fd_failed"))
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

        renderer = UiRenderer(this) { action, needsFilePicker ->
            if (action == "kotlin_image_pick_dir") {
                pickDirLauncher.launch(null)
                return@UiRenderer
            }

            if (needsFilePicker) {
                pendingActionAfterPicker = action
                pickFileLauncher.launch(arrayOf("*/*"))
            } else {
                if (action == "reset") {
                    selectedOutputDir = null
                }
                refreshUi(action)
            }
        }

        // Initial Load
        refreshUi("init")
    }

    private fun refreshUi(action: String, extras: Map<String, Any?> = emptyMap()) {
        lifecycleScope.launch {
            val command = JSONObject().apply {
                put("action", action)
                extras.forEach { (k, v) ->
                    // JSONObject handles proper escaping; null maps to JSON null
                    put(k, v)
                }
            }

            val newUiJson = withContext(Dispatchers.IO) {
                dispatch(command.toString())
            }

            val rootView = renderer.render(newUiJson)
            setContentView(rootView)
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

    external fun dispatch(input: String): String

    companion object {
        init {
            System.loadLibrary("kistaverk_core")
        }
    }
}
