package aeska.kistaverk

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import kotlin.io.DEFAULT_BUFFER_SIZE
import java.io.File
import java.io.FileOutputStream

class MainActivity : ComponentActivity() {

    private lateinit var renderer: UiRenderer
    private var pendingActionAfterPicker: String? = null

    private val pickFileLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        val action = pendingActionAfterPicker
        pendingActionAfterPicker = null

        if (uri == null || action == null) return@registerForActivityResult

        val copied = copyUriToCache(uri)
        if (copied != null) {
            refreshUi(action, mapOf("path" to copied.absolutePath))
        } else {
            // Notify Rust about the failure so it can surface an error state
            refreshUi(action, mapOf<String, Any?>("path" to JSONObject.NULL, "error" to "copy_failed"))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        renderer = UiRenderer(this) { action, needsFilePicker ->
            if (needsFilePicker) {
                pendingActionAfterPicker = action
                pickFileLauncher.launch(arrayOf("*/*"))
            } else {
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

    private fun copyUriToCache(uri: Uri): File? {
        return try {
            val fileName = uri.lastPathSegment?.substringAfterLast('/') ?: "selected.bin"
            val dst = File(cacheDir, "picked_${System.currentTimeMillis()}_${fileName}")
            contentResolver.openInputStream(uri)?.use { input ->
                FileOutputStream(dst).use { output ->
                    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                    while (true) {
                        val read = input.read(buffer)
                        if (read <= 0) break
                        output.write(buffer, 0, read)
                    }
                }
            } ?: return null
            dst
        } catch (_: Exception) {
            null
        }
    }

    external fun dispatch(input: String): String

    companion object {
        init {
            System.loadLibrary("kistaverk_core")
        }
    }
}
