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

class MainActivity : ComponentActivity() {

    private lateinit var renderer: UiRenderer
    private var pendingActionAfterPicker: String? = null

    private val pickFileLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        val action = pendingActionAfterPicker
        pendingActionAfterPicker = null

        if (uri == null || action == null) return@registerForActivityResult

        val fd = openFdForUri(uri)
        if (fd != null) {
            refreshUi(action, mapOf("fd" to fd))
        } else {
            // Notify Rust about the failure so it can surface an error state
            refreshUi(action, mapOf<String, Any?>("fd" to JSONObject.NULL, "error" to "open_fd_failed"))
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

    private fun openFdForUri(uri: Uri): Int? {
        return try {
            contentResolver.openFileDescriptor(uri, "r")?.use { pfd ->
                pfd.detachFd().takeIf { it >= 0 }
            }
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
