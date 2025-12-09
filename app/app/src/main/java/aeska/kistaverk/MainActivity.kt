package aeska.kistaverk

import android.net.Uri
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.wifi.WifiManager
import android.os.Bundle
import android.os.Environment
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Intent
import android.content.IntentFilter
import android.content.Context
import android.util.Base64
import android.os.StatFs
import android.os.BatteryManager
import androidx.annotation.VisibleForTesting
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import aeska.kistaverk.features.ConversionResult
import aeska.kistaverk.features.KotlinImageConversion
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import java.io.File
import java.io.FileDescriptor
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.InputStream
import java.io.OutputStream
import java.io.OutputStreamWriter

class MainActivity : ComponentActivity() {

    private val maxSnapshotSize = 200_000 // guard against TransactionTooLarge
    private lateinit var renderer: UiRenderer
    private lateinit var sensors: AppSensorManager
    private lateinit var cameraManager: CameraManager
    private var pendingActionAfterPicker: String? = null
    private var pendingBindingsAfterPicker: Map<String, String> = emptyMap()
    private var selectedOutputDir: Uri? = null
    private var pdfSourceUri: Uri? = null
    private var rootContainer: FrameLayout? = null
    private var contentHolder: FrameLayout? = null
    private var overlayView: View? = null
    private var lastResult: String? = null
    private var lastFileOutputPath: String? = null
    private var lastFileOutputMime: String? = null
    private var autoRefreshJob: Job? = null
    private val snapshotKey = "rust_snapshot"

    private val pickFileLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        val action = pendingActionAfterPicker
        val bindings = pendingBindingsAfterPicker
        pendingActionAfterPicker = null
        pendingBindingsAfterPicker = emptyMap()
        handlePickerResult(action, uri, bindings)
    }

    private fun cacheLastResult(json: String) {
        val obj = runCatching { JSONObject(json) }.getOrNull() ?: return
        lastFileOutputPath = null
        lastFileOutputMime = null
        fun findResult(o: JSONObject): String? {
            val t = o.optString("text", "")
            if (t.startsWith("Result")) {
                return t.removePrefix("Result").trim().trimStart(':').trim()
            }
            val children = o.optJSONArray("children") ?: return null
            for (i in 0 until children.length()) {
                val maybe = findResult(children.getJSONObject(i))
                if (maybe != null) return maybe
            }
            return null
        }
        fun findOutputPath(o: JSONObject) {
            val t = o.optString("text", "")
            if (t.startsWith("Result saved to:")) {
                val path = t.removePrefix("Result saved to:").trim()
                if (path.isNotEmpty()) {
                    lastFileOutputPath = path
                    lastFileOutputMime = guessMimeFromPath(path)
                }
            } else if (t.startsWith("Path:")) {
                val path = t.removePrefix("Path:").trim()
                if (path.isNotEmpty()) {
                    lastFileOutputPath = path
                    lastFileOutputMime = guessMimeFromPath(path)
                }
            }
            val children = o.optJSONArray("children") ?: return
            for (i in 0 until children.length()) {
                findOutputPath(children.getJSONObject(i))
            }
        }
        lastResult = findResult(obj)
        findOutputPath(obj)
    }

    private fun updateSensorSubscriptions(json: String) {
        sensors.updateSubscriptions(json)
    }

    private fun guessMimeFromPath(path: String): String? {
        return when {
            path.lowercase(Locale.US).endsWith(".pdf") -> "application/pdf"
            path.lowercase(Locale.US).endsWith(".gz") -> "application/gzip"
            path.lowercase(Locale.US).endsWith(".age") -> "application/age-encryption"
            path.lowercase(Locale.US).endsWith(".png") -> "image/png"
            path.lowercase(Locale.US).endsWith(".webp") -> "image/webp"
            path.lowercase(Locale.US).endsWith(".jpg") || path.lowercase(Locale.US).endsWith(".jpeg") -> "image/jpeg"
            else -> null
        }
    }

    private fun copyResultToClipboard() {
        val text = lastResult ?: return
        val cm = getSystemService(CLIPBOARD_SERVICE) as? ClipboardManager ?: return
        cm.setPrimaryClip(ClipData.newPlainText("text_tools_result", text))
    }

    @VisibleForTesting
    fun handlePickerResultForTest(
        action: String?,
        uri: Uri?,
        bindings: Map<String, String>
    ): Boolean = handlePickerResult(action, uri, bindings)

    private fun readClipboardText(maxLen: Int = 256): String? {
        val cm = getSystemService(CLIPBOARD_SERVICE) as? ClipboardManager ?: return null
        val clip = cm.primaryClip ?: return null
        val text = clip.getItemAt(0)?.coerceToText(this)?.toString()?.trim() ?: return null
        return text.takeIf { it.length <= maxLen }
    }

    private fun readHexFromClipboard(): String? {
        val cm = getSystemService(CLIPBOARD_SERVICE) as? ClipboardManager ?: return null
        val clip = cm.primaryClip ?: return null
        val text = clip.getItemAt(0)?.coerceToText(this)?.toString()?.trim() ?: return null
        val normalized = text.removePrefix("#")
        val regex = Regex("^[0-9a-fA-F]{6}$")
        return if (regex.matches(normalized)) "#$normalized" else null
    }

    private fun shareResult() {
        val text = lastResult ?: return
        val intent = Intent(Intent.ACTION_SEND).apply {
            type = "text/plain"
            putExtra(Intent.EXTRA_TEXT, text)
        }
        startActivity(Intent.createChooser(intent, "Share result"))
    }

    private fun launchSaveAs(sourcePath: String?, mime: String) {
        if (sourcePath == null) return
        val suggested = runCatching { File(sourcePath).name.takeIf { it.isNotBlank() } }
            .getOrNull()
            ?: "output"
        pendingSaveSourcePath = sourcePath
        pendingSaveMime = mime
        pendingSaveSuggested = suggested
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = mime
            putExtra(Intent.EXTRA_TITLE, suggested)
        }
        saveAsLauncher.launch(intent)
    }

    private fun copyFileToUri(sourcePath: String, target: Uri, mime: String?) {
        val input: InputStream = if (sourcePath.startsWith("content://")) {
            contentResolver.openInputStream(Uri.parse(sourcePath)) ?: return
        } else {
            val file = File(sourcePath)
            if (!file.exists()) return
            FileInputStream(file)
        }
        input.use { inp ->
            val out: OutputStream = contentResolver.openOutputStream(target, "w") ?: return
            out.use { dst ->
                val buf = ByteArray(8 * 1024)
                while (true) {
                    val read = inp.read(buf)
                    if (read <= 0) break
                    dst.write(buf, 0, read)
                }
                dst.flush()
            }
        }
        if (mime != null) {
            try {
                contentResolver.takePersistableUriPermission(
                    target,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                )
            } catch (_: Exception) {
                // best effort
            }
        }
    }

    private fun startCompass() = sensors.startCompass()
    private fun stopCompass() = sensors.stopCompass()
    private fun startBarometer() = sensors.startBarometer()
    private fun stopBarometer() = sensors.stopBarometer()
    private fun startMagnetometer() = sensors.startMagnetometer()
    private fun stopMagnetometer() = sensors.stopMagnetometer()

    private fun handleTextFind(action: String, bindings: Map<String, String>) {
        val direction = when (action) {
            "text_viewer_find_next" -> "next"
            "text_viewer_find_prev" -> "prev"
            else -> null
        }
        val query = when (action) {
            "text_viewer_find_clear" -> ""
            else -> bindings["find_query"].orEmpty()
        }
        renderer.performTextFind(query, direction)
        // Sync to Rust without forcing a re-render
        lifecycleScope.launch(Dispatchers.IO) {
            val cmd = JSONObject().apply {
                put("action", "text_viewer_find")
                val b = JSONObject()
                b.put("find_query", query)
                direction?.let { b.put("find_direction", it) }
                put("bindings", b)
            }
            dispatch(cmd.toString())
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

    private var pendingSaveSourcePath: String? = null
    private var pendingSaveMime: String? = null
    private var pendingSaveSuggested: String? = null
    private val saveAsLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val uri = result.data?.data
        val sourcePath = pendingSaveSourcePath
        val mime = pendingSaveMime
        pendingSaveSourcePath = null
        pendingSaveMime = null
        pendingSaveSuggested = null
        if (uri == null || sourcePath == null) return@registerForActivityResult
        copyFileToUri(sourcePath, uri, mime)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val entry = resolveEntry(intent)

        sensors = AppSensorManager(
            activity = this,
            scope = lifecycleScope,
            refreshUi = { action, bindings -> refreshUi(action, bindings = bindings) },
            dispatchRaw = { command -> dispatch(command) }
        )
        cameraManager = CameraManager(
            activity = this,
            processFrame = ::processQrCameraFrame,
            dispatchAction = { action, bindings -> dispatchWithOptionalLoading(action, bindings = bindings) }
        )

        renderer = UiRenderer(this) { action, needsFilePicker, bindings ->
            if (KotlinImageConversion.isConversionAction(action)) {
                val sourcePath = bindings["image_source_path"]
                if (sourcePath != null) {
                    val uri = Uri.fromFile(File(sourcePath))
                    handleKotlinImageConversion(uri, action, bindings)
                } else {
                    // Fallback to Rust if source path missing (should not happen if UI is correct)
                    dispatchWithOptionalLoading(action, bindings = bindings)
                }
                return@UiRenderer
            }
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
            if (action == "text_tools_share_result") {
                shareResult()
                return@UiRenderer
            }
            if (action == "text_tools_copy_to_input") {
                copyResultToClipboard()
            }
            if (action == "color_copy_clipboard") {
                copyResultToClipboard()
                return@UiRenderer
            }
            if (action == "color_copy_hex_input") {
                val fromClipboard = readHexFromClipboard()
                val merged = if (fromClipboard != null) {
                    bindings + mapOf("color_input" to fromClipboard)
                } else bindings
                dispatchWithOptionalLoading(action, bindings = merged)
                return@UiRenderer
            }
            if (action == "pdf_extract" || action == "pdf_delete" || action == "pdf_sign") {
                dispatchPdfAction(action, bindings)
                return@UiRenderer
            }
            if (action == "pdf_set_title") {
                dispatchPdfAction(action, bindings)
                return@UiRenderer
            }
            if (action == "pdf_save_as") {
                launchSaveAs(lastFileOutputPath, lastFileOutputMime ?: "application/pdf")
                return@UiRenderer
            }
            if (action == "sensor_logger_start") {
                startSensorLogging(bindings)
                return@UiRenderer
            }
            if (action == "sensor_logger_stop") {
                stopSensorLogging()
                return@UiRenderer
            }
            if (action == "text_viewer_find_submit" || action == "text_viewer_find_next" || action == "text_viewer_find_prev" || action == "text_viewer_find_clear") {
                handleTextFind(action, bindings)
                return@UiRenderer
            }
            if (action == "barometer_screen") {
                startBarometer()
            }
            if (action == "magnetometer_screen") {
                startMagnetometer()
            }
            if (action == "sensor_logger_share") {
                shareLastLog()
                return@UiRenderer
            }
            if (action == "kotlin_image_save_as") {
                val mime = lastFileOutputMime ?: "image/*"
                launchSaveAs(lastFileOutputPath, mime)
                return@UiRenderer
            }
            if (action == "gzip_save_as") {
                val mime = lastFileOutputMime ?: "application/gzip"
                launchSaveAs(lastFileOutputPath, mime)
                return@UiRenderer
            }
            if (action == "vault_save_as") {
                val mime = lastFileOutputMime ?: "application/octet-stream"
                launchSaveAs(lastFileOutputPath, mime)
                return@UiRenderer
            }
            if (action == "system_info_update") {
                val bindingsWithMetrics = bindings + collectSystemInfoBindings()
                dispatchWithOptionalLoading(action, bindings = bindingsWithMetrics)
                return@UiRenderer
            }
            if (action == "system_info_screen") {
                dispatchWithOptionalLoading(action, bindings = bindings)
                val metrics = collectSystemInfoBindings()
                if (metrics.isNotEmpty()) {
                    dispatchWithOptionalLoading("system_info_update", bindings = metrics)
                }
                return@UiRenderer
            }

            if (needsFilePicker) {
                pendingActionAfterPicker = action
                pendingBindingsAfterPicker = bindings
                val mimeTypes = when {
                    action.startsWith("pdf_") -> arrayOf("application/pdf")
                    action == "text_viewer_open" -> arrayOf("text/*", "text/plain", "text/csv", "application/csv")
                    else -> arrayOf("*/*")
                }
                pickFileLauncher.launch(mimeTypes)
            } else {
                if (action == "reset") {
                    selectedOutputDir = null
                    pdfSourceUri = null
                    lastFileOutputPath = null
                    lastFileOutputMime = null
                    stopSensorLogging()
                    stopCompass()
                }
                dispatchWithOptionalLoading(action, bindings = bindings)
            }
        }

        val restoredSnapshot = savedInstanceState?.getString(snapshotKey)
        val handledViewIntent = handleViewIntent(intent)
        if (restoredSnapshot != null && !handledViewIntent) {
            lifecycleScope.launch { restoreSnapshotAndRender(restoredSnapshot) }
        } else if (!handledViewIntent) {
            val initialAction = if (entry == "pdf_signature") "pdf_tools_screen" else "init"
            refreshUi(initialAction)
        }

        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    stopSensorLogging()
                    stopQrScanner() // Stop QR scanner when back is pressed
                    refreshUi("back")
                }
            }
        )
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        val snapshot = runBlocking {
            withContext(Dispatchers.IO) { requestSnapshot() }
        }
        if (snapshot != null && snapshot.length <= maxSnapshotSize) {
            outState.putString(snapshotKey, snapshot)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        sensors.onDestroy()
        cameraManager.stopQrScanner()
    }

    override fun onPause() {
        super.onPause()
        cameraManager.stopQrScanner()
    }

    override fun onResume() {
        super.onResume()
        // If current screen is QrReceive and permission is granted, restart scanner
        lifecycleScope.launch(Dispatchers.IO) { // Dispatch to IO thread for Rust call
            val currentScreenJson = dispatch(JSONObject().apply { put("action", "snapshot_screen_only") }.toString())
            withContext(Dispatchers.Main) {
                val currentScreenId = JSONObject(currentScreenJson).optString("id")
                if (currentScreenId == "QrReceiveScreen") {
                    cameraManager.onScreenChanged(true, contentHolder)
                }
            }
        }
    }

    @Deprecated("Android is migrating to ActivityResult APIs; kept for legacy permission callback")
    @Suppress("DEPRECATION")
    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == PERMISSION_LOCATION) {
            sensors.onPermissionResult(requestCode, grantResults, emptyMap())
        } else if (requestCode == CAMERA_PERMISSION_REQUEST_CODE) {
            cameraManager.onPermissionResult(grantResults)
        }
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        val entry = resolveEntry(intent)
        if (entry == "pdf_signature") {
            refreshUi("pdf_tools_screen")
            return
        }
        val handled = handleViewIntent(intent)
        if (!handled) {
            // fallback: no special handling
        }
    }

    private val skipNativeLoad = System.getProperty("kistaverk.skipNativeLoad") == "true"

    private fun collectSystemInfoBindings(): Map<String, String> {
        val result = mutableMapOf<String, String>()
        val storage = readStorageStats()
        storage?.totalBytes?.let { result["storage_total_bytes"] = it.toString() }
        storage?.freeBytes?.let { result["storage_free_bytes"] = it.toString() }

        val network = readNetworkInfo()
        network.connection?.let { result["network_connection"] = it }
        network.ssid?.let { result["network_ssid"] = it }
        network.ip?.let { result["network_ip"] = it }

        val battery = readBatteryInfo()
        battery.level?.let { result["battery_level_pct"] = it.toString() }
        battery.status?.let { result["battery_status"] = it }

        result["device_manufacturer"] = android.os.Build.MANUFACTURER ?: ""
        result["device_model"] = android.os.Build.MODEL ?: ""
        result["device_os_version"] = android.os.Build.VERSION.RELEASE ?: ""

        result["timestamp"] = isoTimestamp()
        return result.filterValues { it.isNotEmpty() }
    }

    private data class StorageStats(val totalBytes: Long, val freeBytes: Long)

    private fun readStorageStats(): StorageStats? {
        return runCatching {
            val dir = filesDir ?: return null
            val stat = StatFs(dir.absolutePath)
            StorageStats(stat.totalBytes, stat.availableBytes)
        }.getOrNull()
    }

    private data class NetworkSnapshot(val connection: String?, val ssid: String?, val ip: String?)

    private fun readNetworkInfo(): NetworkSnapshot {
        val cm = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
        val active = cm?.activeNetwork
        val caps = active?.let { cm.getNetworkCapabilities(it) }
        val lp = active?.let { cm.getLinkProperties(it) }

        val connection = when {
            caps?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true -> "wifi"
            caps?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true -> "cellular"
            caps?.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) == true -> "ethernet"
            caps != null -> "online"
            else -> "offline"
        }

        val ip = lp?.linkAddresses
            ?.firstOrNull { !it.address.isLoopbackAddress }
            ?.address
            ?.hostAddress

        val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
        val ssidResult = runCatching { currentSsid(wifiManager) }
        if (ssidResult.isFailure) {
            showNetworkPermissionToastOnce()
        }
        val ssid = ssidResult.getOrNull()
            ?.takeIf { it.isNotBlank() && it != "<unknown ssid>" }
            ?.trim('"')

        return NetworkSnapshot(connection = connection, ssid = ssid, ip = ip)
    }

    private var networkToastShown = false
    private fun showNetworkPermissionToastOnce() {
        if (networkToastShown) return
        networkToastShown = true
        runOnUiThread {
            Toast.makeText(
                this,
                "Network details unavailable (permission or state).",
                Toast.LENGTH_SHORT
            ).show()
        }
    }

    @Suppress("DEPRECATION")
    private fun currentSsid(wifiManager: WifiManager?): String? = wifiManager?.connectionInfo?.ssid

    private data class BatterySnapshot(val level: Int?, val status: String?)

    private fun readBatteryInfo(): BatterySnapshot {
        val intent = registerReceiver(null, android.content.IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val level = intent?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = intent?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        val pct = if (level >= 0 && scale > 0) ((level * 100f) / scale).toInt() else null
        val statusCode = intent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val status = when (statusCode) {
            BatteryManager.BATTERY_STATUS_CHARGING -> "charging"
            BatteryManager.BATTERY_STATUS_DISCHARGING -> "discharging"
            BatteryManager.BATTERY_STATUS_FULL -> "full"
            BatteryManager.BATTERY_STATUS_NOT_CHARGING -> "not_charging"
            else -> null
        }
        return BatterySnapshot(level = pct, status = status)
    }

    private fun isoTimestamp(): String {
        return try {
            val fmt = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss'Z'", Locale.US)
            fmt.format(Date())
        } catch (_: Exception) {
            System.currentTimeMillis().toString()
        }
    }

    internal fun refreshUi(
        action: String,
        extras: Map<String, Any?> = emptyMap(),
        bindings: Map<String, String> = emptyMap(),
        loadingOnly: Boolean = false
    ) {
        if (skipNativeLoad) {
            val mergedBindings = bindings.toMutableMap()
            readClipboardText()?.let { clip ->
                mergedBindings.putIfAbsent("clipboard", clip)
            }
            val command = JSONObject().apply {
                put("action", action)
                extras.forEach { (k, v) -> put(k, v) }
                if (mergedBindings.isNotEmpty()) {
                    val bindingsObj = JSONObject()
                    mergedBindings.forEach { (k, v) -> bindingsObj.put(k, v) }
                    put("bindings", bindingsObj)
                }
                if (loadingOnly) {
                    put("loading_only", true)
                }
            }
            val newUiJson = dispatch(command.toString())
            if (loadingOnly) {
                showOverlay(command.optString("action", "Working..."))
            } else {
                val rootView = runCatching { renderer.render(newUiJson) }
                    .getOrElse { throwable ->
                        renderer.renderFallback(
                            title = "Render error",
                            message = throwable.message ?: "unknown_render_error"
                        )
                    }
                attachContent(rootView)
                hideOverlay()
                cacheLastResult(newUiJson)
                updateSensorSubscriptions(newUiJson)
                scheduleAutoRefresh(newUiJson)

                val currentScreen = JSONObject(newUiJson).optJSONObject("layout")?.optString("id")
                cameraManager.onScreenChanged(currentScreen == "QrReceiveScreen", contentHolder)
            }
        } else {
            lifecycleScope.launch {
                val mergedBindings = bindings.toMutableMap()
                readClipboardText()?.let { clip ->
                    mergedBindings.putIfAbsent("clipboard", clip)
                }
                val command = JSONObject().apply {
                    put("action", action)
                    extras.forEach { (k, v) -> put(k, v) }
                    if (mergedBindings.isNotEmpty()) {
                        val bindingsObj = JSONObject()
                        mergedBindings.forEach { (k, v) -> bindingsObj.put(k, v) }
                        put("bindings", bindingsObj)
                    }
                    if (loadingOnly) {
                        put("loading_only", true)
                    }
                }

                val newUiJson = withContext(Dispatchers.IO) { dispatch(command.toString()) }
                if (loadingOnly) {
                    withContext(Dispatchers.Main) {
                        showOverlay(command.optString("action", "Working..."))
                    }
                } else {
                    withContext(Dispatchers.Main) {
                        val rootView = runCatching { renderer.render(newUiJson) }
                            .getOrElse { throwable ->
                                renderer.renderFallback(
                                    title = "Render error",
                                    message = throwable.message ?: "unknown_render_error"
                                )
                            }
                        attachContent(rootView)
                        hideOverlay()
                        cacheLastResult(newUiJson)
                        updateSensorSubscriptions(newUiJson)
                        scheduleAutoRefresh(newUiJson)

                        // Check current screen from newUiJson and manage QR scanner lifecycle
                        val currentScreen = JSONObject(newUiJson).optJSONObject("layout")?.optString("id")
                        if (currentScreen == "QrReceiveScreen") { // Assuming Rust sets screen ID
                             // Lazily add the PreviewView if not already there
                            cameraManager.onScreenChanged(true, contentHolder)
                        } else {
                            cameraManager.onScreenChanged(false, contentHolder)
                        }
                    }
                }
            }
        }
    }

    private fun scheduleAutoRefresh(json: String) {
        val obj = runCatching { JSONObject(json) }.getOrNull() ?: return
        val interval = obj.optLong("auto_refresh_ms", 0L)
        val action = obj.optString("auto_refresh_action", "")
        autoRefreshJob?.cancel()
        if (interval <= 0 || action.isBlank()) return
        autoRefreshJob = lifecycleScope.launch {
            delay(interval)
            dispatchWithOptionalLoading(action)
        }
    }

    private suspend fun restoreSnapshotAndRender(snapshot: String) {
        val command = JSONObject().apply {
            put("action", "restore_state")
            put("snapshot", snapshot)
        }

        val newUiJson = withContext(Dispatchers.IO) {
            dispatch(command.toString())
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
        cacheLastResult(newUiJson)
    }

    private fun requestSnapshot(): String? {
        val command = JSONObject().apply {
            put("action", "snapshot")
        }
        val json = dispatch(command.toString())
        val obj = runCatching { JSONObject(json) }.getOrNull() ?: return null
        return obj.optString("snapshot").takeIf { it.isNotEmpty() }
    }

    private fun openFdForUri(uri: Uri): Int? {
        return try {
            contentResolver.openFileDescriptor(uri, "r")?.use { pfd ->
                val detached: Int? = runCatching { pfd.detachFd() }.getOrNull()?.takeIf { it >= 0 }
                val fallback: Int? = pfd.fileDescriptor?.let { fd ->
                    runCatching {
                        val descriptor = FileDescriptor::class.java.getDeclaredField("descriptor")
                        descriptor.isAccessible = true
                        descriptor.getInt(fd)
                    }.getOrNull()?.takeIf { it >= 0 }
                }
                detached ?: fallback
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun readBytes(uri: Uri): ByteArray? {
        return try {
            contentResolver.openInputStream(uri)?.use { it.readBytes() }
        } catch (_: Exception) {
            null
        }
    }

    private fun startQrScanner() = cameraManager.startQrScanner()
    private fun stopQrScanner() = cameraManager.stopQrScanner()
    private fun resolveEntry(intent: Intent?): String? {
        if (intent == null) return null
        val explicit = intent.getStringExtra("entry")
        if (!explicit.isNullOrEmpty()) return explicit
        if (intent.action == "aeska.kistaverk.PDF_SIGN") return "pdf_signature"
        val componentName = intent.component?.className.orEmpty()
        if (componentName.endsWith("PdfSignLauncher")) return "pdf_signature"
        return null
    }

    private fun handlePickerResult(
        actionInput: String?,
        uri: Uri?,
        bindings: Map<String, String>
    ): Boolean {
        var action = actionInput ?: return false
        if (uri == null) return false

        if (KotlinImageConversion.isConversionAction(action)) {
            handleKotlinImageConversion(uri, action, bindings)
            return true
        }

        if (action == "pdf_select") {
            pdfSourceUri = uri
        }

        if (action == "pdf_signature_load") {
            val bytes = readBytes(uri)
            val b64 = bytes?.let { Base64.encodeToString(it, Base64.NO_WRAP) }
            if (b64 != null) {
                refreshUi(
                    "pdf_signature_store",
                    bindings = mapOf("signature_base64" to b64)
                )
            } else {
                refreshUi(
                    "pdf_signature_store",
                    bindings = emptyMap(),
                    extras = mapOf("error" to "signature_load_failed")
                )
            }
            return true
        }

        val fd = openFdForUri(uri)
        val extras = mutableMapOf<String, Any?>()
        if (fd != null) {
            extras["fd"] = fd
        } else {
            extras["fd"] = JSONObject.NULL
            extras["error"] = "open_fd_failed"
        }
        if (action.startsWith("pdf_")) {
            extras["path"] = uri.toString()
            if (action == "pdf_merge") {
                val primaryUri = pdfSourceUri
                if (primaryUri != null) {
                    extras["primary_path"] = primaryUri.toString()
                    val pfd = openFdForUri(primaryUri)
                    if (pfd != null) {
                        extras["primary_fd"] = pfd
                    }
                } else {
                    extras["error"] = "select_pdf_first"
                }
            }
        } else {
            extras["path"] = uri.toString()
        }

        if (action == "text_viewer_screen") {
            // File picker from menu should directly open the file in the viewer.
            action = "text_viewer_open"
        }

        dispatchWithOptionalLoading(
            action = action,
            bindings = bindings,
            extras = extras
        )
        return true
    }

    private fun handleViewIntent(intent: Intent?): Boolean {
        val data = intent?.data ?: return false
        val action = intent.action ?: return false
        if (action != Intent.ACTION_VIEW) return false
        val fd = openFdForUri(data)
        val extras = mutableMapOf<String, Any?>()
        extras["path"] = data.toString()
        if (fd != null) {
            extras["fd"] = fd
        } else {
            extras["error"] = "open_fd_failed"
        }
        refreshUi("text_viewer_open", extras = extras)
        return true
    }

    private fun startSensorLogging(bindings: Map<String, String>) = sensors.startLogging(bindings)
    private fun stopSensorLogging() = sensors.stopLogging()
    private fun shareLastLog() = sensors.shareLastLog()

    private fun dispatchPdfAction(action: String, bindings: Map<String, String>) {
        val uri = pdfSourceUri
        if (uri == null) {
            refreshUi(
                "pdf_select",
                bindings = bindings,
                extras = mapOf("error" to "select_pdf_first")
            )
            return
        }

        val fd = openFdForUri(uri)
        val extras = mutableMapOf<String, Any?>(
            "path" to uri.toString()
        )
        if (fd != null) {
            extras["fd"] = fd
        } else {
            extras["fd"] = JSONObject.NULL
            extras["error"] = "open_fd_failed"
        }
        dispatchWithOptionalLoading(action, bindings = bindings, extras = extras)
    }

    private fun handleKotlinImageConversion(uri: Uri, action: String, bindings: Map<String, String>) {
        lifecycleScope.launch {
            val result = withContext(Dispatchers.IO) {
                if (action == "kotlin_image_resize") {
                    KotlinImageConversion.resize(
                        context = this@MainActivity,
                        cacheDir = cacheDir,
                        picturesDir = getExternalFilesDir(Environment.DIRECTORY_PICTURES),
                        outputDirUri = selectedOutputDir,
                        uri = uri,
                        bindings = bindings
                    )
                } else {
                    KotlinImageConversion.convert(
                        context = this@MainActivity,
                        cacheDir = cacheDir,
                        picturesDir = getExternalFilesDir(Environment.DIRECTORY_PICTURES),
                        outputDirUri = selectedOutputDir,
                        uri = uri,
                        action = action,
                        bindings = bindings
                    )
                }
            }

            val echoedBindings = bindings.toMutableMap()

            when (result) {
                is ConversionResult.Success -> {
                    result.scalePercent?.let { echoedBindings["resize_scale_pct"] = it.toString() }
                    result.quality?.let { echoedBindings["resize_quality"] = it.toString() }
                    result.targetBytes?.let { echoedBindings["resize_target_kb"] = (it / 1024).toString() }
                    echoedBindings["resize_use_webp"] = (result.target.key == "webp").toString()

                    refreshUi(
                        "kotlin_image_result",
                        mapOf(
                            "target" to result.target.key,
                            "result_path" to result.destination,
                            "result_size" to result.size,
                            "result_format" to result.format
                        ),
                        bindings = echoedBindings
                    )
                }
                is ConversionResult.Failure -> {
                    val reason = result.reason ?: "conversion_failed"
                    refreshUi(
                        "kotlin_image_result",
                        mapOf(
                            "target" to (result.target?.key ?: JSONObject.NULL),
                            "error" to reason
                        ),
                        bindings = echoedBindings
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
        val isHashAction = action.startsWith("hash_file_") || action == "hash_all"
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
        val holder = contentHolder ?: return
        if (holder.childCount == 1 && holder.getChildAt(0) === view) {
            return
        }
        if (view.parent != null && view.parent !== holder) {
            (view.parent as? ViewGroup)?.removeView(view)
        }
        holder.removeAllViews()
        holder.addView(view)
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
    external fun processQrCameraFrame(
        lumaData: ByteArray,
        width: Int,
        height: Int,
        rowStride: Int,
        rotationDeg: Int
    ): String?

    companion object {
        init {
            if (System.getProperty("kistaverk.skipNativeLoad") != "true") {
                System.loadLibrary("kistaverk_core")
            }
        }

        // Arbitrary request code for location permission prompts
        internal const val PERMISSION_LOCATION = 1001
        internal const val CAMERA_PERMISSION_REQUEST_CODE = 2001
    }
}
