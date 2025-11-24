package aeska.kistaverk

import android.net.Uri
import android.os.Bundle
import android.os.Environment
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Intent
import android.util.Base64
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import android.os.Handler
import android.os.HandlerThread
import android.content.pm.PackageManager
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import androidx.core.content.ContextCompat
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.OnBackPressedCallback
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
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStreamWriter
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.Date

class MainActivity : ComponentActivity() {

    private lateinit var renderer: UiRenderer
    private var sensorManager: SensorManager? = null
    private var sensorThread: HandlerThread? = null
    private var sensorHandler: Handler? = null
    private var sensorListener: SensorEventListener? = null
    private var logFile: File? = null
    private var logWriter: OutputStreamWriter? = null
    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null
    private var pendingSensorStart = false
    private var pendingActionAfterPicker: String? = null
    private var pendingBindingsAfterPicker: Map<String, String> = emptyMap()
    private var selectedOutputDir: Uri? = null
    private var pdfSourceUri: Uri? = null
    private var rootContainer: FrameLayout? = null
    private var contentHolder: FrameLayout? = null
    private var overlayView: View? = null
    private var lastResult: String? = null
    private var lastSensorLogPath: String? = null
    private val snapshotKey = "rust_snapshot"

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
            return@registerForActivityResult
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
        }

        dispatchWithOptionalLoading(
            action = action,
            bindings = bindings,
            extras = extras
        )
    }

    private fun cacheLastResult(json: String) {
        val obj = runCatching { JSONObject(json) }.getOrNull() ?: return
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
        lastResult = findResult(obj)
    }

    private fun copyResultToClipboard() {
        val text = lastResult ?: return
        val cm = getSystemService(CLIPBOARD_SERVICE) as? ClipboardManager ?: return
        cm.setPrimaryClip(ClipData.newPlainText("text_tools_result", text))
    }

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

        val entry = resolveEntry(intent)

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
            if (action == "sensor_logger_start") {
                startSensorLogging()
                return@UiRenderer
            }
            if (action == "sensor_logger_stop") {
                stopSensorLogging()
                return@UiRenderer
            }
            if (action == "sensor_logger_share") {
                shareLastLog()
                return@UiRenderer
            }

            if (needsFilePicker) {
                pendingActionAfterPicker = action
                pendingBindingsAfterPicker = bindings
                val mimeTypes = if (action.startsWith("pdf_")) arrayOf("application/pdf") else arrayOf("*/*")
                pickFileLauncher.launch(mimeTypes)
            } else {
                if (action == "reset") {
                    selectedOutputDir = null
                    pdfSourceUri = null
                    stopSensorLogging()
                }
                dispatchWithOptionalLoading(action, bindings = bindings)
            }
        }

        val restoredSnapshot = savedInstanceState?.getString(snapshotKey)
        if (restoredSnapshot != null) {
            lifecycleScope.launch { restoreSnapshotAndRender(restoredSnapshot) }
        } else {
            val initialAction = if (entry == "pdf_signature") "pdf_tools_screen" else "init"
            refreshUi(initialAction)
        }

        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    stopSensorLogging()
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
        if (snapshot != null) {
            outState.putString(snapshotKey, snapshot)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopSensorLogging()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == PERMISSION_LOCATION) {
            val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
            if (granted && pendingSensorStart) {
                pendingSensorStart = false
                startSensorLogging()
            } else {
                val bindings = mutableMapOf<String, String>()
                bindings["sensor_status"] = "location permission denied"
                refreshUi("sensor_logger_status", bindings = bindings)
            }
        }
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        val entry = resolveEntry(intent)
        if (entry == "pdf_signature") {
            refreshUi("pdf_tools_screen")
        }
    }

    private fun refreshUi(
        action: String,
        extras: Map<String, Any?> = emptyMap(),
        bindings: Map<String, String> = emptyMap(),
        loadingOnly: Boolean = false
    ) {
        lifecycleScope.launch {
            val mergedBindings = bindings.toMutableMap()
            readClipboardText()?.let { clip ->
                mergedBindings.putIfAbsent("clipboard", clip)
            }
            val command = JSONObject().apply {
                put("action", action)
                extras.forEach { (k, v) ->
                    // JSONObject handles proper escaping; null maps to JSON null
                    put(k, v)
                }
                if (mergedBindings.isNotEmpty()) {
                    val bindingsObj = JSONObject()
                    mergedBindings.forEach { (k, v) -> bindingsObj.put(k, v) }
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

            cacheLastResult(newUiJson)
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
                pfd.detachFd().takeIf { it >= 0 }
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

    private fun resolveEntry(intent: Intent?): String? {
        if (intent == null) return null
        val explicit = intent.getStringExtra("entry")
        if (!explicit.isNullOrEmpty()) return explicit
        if (intent.action == "aeska.kistaverk.PDF_SIGN") return "pdf_signature"
        val componentName = intent.component?.className.orEmpty()
        if (componentName.endsWith("PdfSignLauncher")) return "pdf_signature"
        return null
    }

    private fun startSensorLogging() {
        if (sensorManager == null) {
            sensorManager = getSystemService(SENSOR_SERVICE) as? SensorManager
        }
        val mgr = sensorManager ?: return
        if (!hasLocationPermission()) {
            pendingSensorStart = true
            requestPermissions(arrayOf(android.Manifest.permission.ACCESS_FINE_LOCATION), PERMISSION_LOCATION)
            return
        }
        stopSensorLogging()

        val thread = HandlerThread("SensorLogger")
        thread.start()
        sensorThread = thread
        sensorHandler = Handler(thread.looper)

        val fmt = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US)
        val fname = "sensors_${fmt.format(Date())}.csv"
        logFile = File(getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS) ?: filesDir, fname)
        val fos = FileOutputStream(logFile!!)
        logWriter = OutputStreamWriter(fos)
        logWriter?.write("timestamp_ms,type,v1,v2,v3,battery_level,battery_voltage\n")

        val batteryStatus = registerReceiver(null, android.content.IntentFilter(Intent.ACTION_BATTERY_CHANGED))

        sensorListener = object : SensorEventListener {
            override fun onSensorChanged(event: SensorEvent) {
                val ts = System.currentTimeMillis()
                val type = when (event.sensor.type) {
                    Sensor.TYPE_ACCELEROMETER -> "ACCEL"
                    Sensor.TYPE_GYROSCOPE -> "GYRO"
                    Sensor.TYPE_MAGNETIC_FIELD -> "MAG"
                    Sensor.TYPE_PRESSURE -> "PRESSURE"
                    else -> return
                }
                val v1 = event.values.getOrNull(0) ?: 0f
                val v2 = event.values.getOrNull(1) ?: 0f
                val v3 = event.values.getOrNull(2) ?: 0f
                val level = batteryStatus?.getIntExtra(android.os.BatteryManager.EXTRA_LEVEL, -1) ?: -1
                val voltage = batteryStatus?.getIntExtra(android.os.BatteryManager.EXTRA_VOLTAGE, -1) ?: -1
                try {
                    logWriter?.write("$ts,$type,$v1,$v2,$v3,$level,$voltage\n")
                    logWriter?.flush()
                    lastSensorLogPath = logFile?.absolutePath
                    val bindings = mutableMapOf<String, String>()
                    bindings["sensor_status"] = "logging"
                    logFile?.absolutePath?.let { bindings["sensor_path"] = it }
                    refreshUi("sensor_logger_status", bindings = bindings)
                } catch (_: Exception) {
                }
            }

            override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit
        }

        val types = listOf(
            Sensor.TYPE_ACCELEROMETER,
            Sensor.TYPE_GYROSCOPE,
            Sensor.TYPE_MAGNETIC_FIELD,
            Sensor.TYPE_PRESSURE
        )
        types.forEach { t ->
            mgr.getDefaultSensor(t)?.let { sensor ->
                mgr.registerListener(sensorListener, sensor, SensorManager.SENSOR_DELAY_GAME, sensorHandler)
            }
        }
        startLocationLogging()
        val bindings = mutableMapOf<String, String>()
        bindings["sensor_status"] = "logging"
        logFile?.absolutePath?.let { bindings["sensor_path"] = it }
        refreshUi("sensor_logger_status", bindings = bindings)
    }

    private fun stopSensorLogging() {
        sensorManager?.unregisterListener(sensorListener)
        sensorListener = null
        sensorHandler = null
        sensorThread?.quitSafely()
        sensorThread = null
        stopLocationLogging()
        try {
            logWriter?.close()
        } catch (_: Exception) {
        }
        logWriter = null
        val bindings = mutableMapOf<String, String>()
        bindings["sensor_status"] = "stopped"
        logFile?.absolutePath?.let { bindings["sensor_path"] = it }
        refreshUi("sensor_logger_status", bindings = bindings)
    }

    private fun shareLastLog() {
        val path = lastSensorLogPath ?: return
        val file = File(path)
        if (!file.exists()) return
        val uri = androidx.core.content.FileProvider.getUriForFile(
            this,
            "$packageName.fileprovider",
            file
        )
        val intent = Intent(Intent.ACTION_SEND).apply {
            type = "text/csv"
            putExtra(Intent.EXTRA_STREAM, uri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        startActivity(Intent.createChooser(intent, "Share sensor log"))
    }

    private fun startLocationLogging() {
        if (locationManager == null) {
            locationManager = getSystemService(LOCATION_SERVICE) as? LocationManager
        }
        val mgr = locationManager ?: return
        val listener = object : LocationListener {
            override fun onLocationChanged(location: Location) {
                val ts = System.currentTimeMillis()
                val lat = location.latitude
                val lon = location.longitude
                val acc = location.accuracy.toDouble()
                val bindings = mutableMapOf<String, String>()
                bindings["sensor_status"] = "logging"
                logFile?.absolutePath?.let { bindings["sensor_path"] = it }
                try {
                    logWriter?.write("$ts,GPS,$lat,$lon,$acc,-1,-1\n")
                    logWriter?.flush()
                    lastSensorLogPath = logFile?.absolutePath
                    refreshUi("sensor_logger_status", bindings = bindings)
                } catch (_: Exception) {
                }
            }

            override fun onStatusChanged(provider: String?, status: Int, extras: android.os.Bundle?) = Unit
            override fun onProviderEnabled(provider: String) = Unit
            override fun onProviderDisabled(provider: String) = Unit
        }
        locationListener = listener
        try {
            mgr.requestLocationUpdates(LocationManager.GPS_PROVIDER, 2000L, 0f, listener, sensorHandler?.looper)
        } catch (_: SecurityException) {
            // permission denied, status already handled elsewhere
        }
    }

    private fun stopLocationLogging() {
        locationManager?.removeUpdates(locationListener ?: return)
        locationListener = null
    }

    private fun hasLocationPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, android.Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED
    }

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
            if (System.getProperty("kistaverk.skipNativeLoad") != "true") {
                System.loadLibrary("kistaverk_core")
            }
        }

        // Arbitrary request code for location permission prompts
        private const val PERMISSION_LOCATION = 1001
    }
}
