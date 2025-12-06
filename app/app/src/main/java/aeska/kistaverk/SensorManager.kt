package aeska.kistaverk

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import android.os.Handler
import android.os.HandlerThread
import android.widget.Toast
import androidx.core.content.ContextCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStreamWriter
import java.util.Locale

/**
 * Handles sensor logging and lightweight real-time sensors (compass, barometer, magnetometer).
 * Keeps MainActivity slim by exposing a small API surface.
 */
class AppSensorManager(
    private val activity: MainActivity,
    private val scope: CoroutineScope,
    private val refreshUi: (String, Map<String, String>) -> Unit,
    private val dispatchRaw: (String) -> Unit
) {
    private var sensorThread: HandlerThread? = null
    private var sensorHandler: Handler? = null
    private var sensorListener: SensorEventListener? = null
    private var logFile: File? = null
    private var logWriter: OutputStreamWriter? = null
    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null
    private var pendingSensorStart = false
    private var pendingSensorBindings: Map<String, String>? = null
    private var lastSensorLogPath: String? = null
    private var lastSensorUiTs: Long = 0L
    @Volatile private var isLogging = false

    private var compassThread: HandlerThread? = null
    private var compassHandler: Handler? = null
    private var compassListener: SensorEventListener? = null
    private var compassSensorManager: SensorManager? = null
    private var compassRotationSensor: Sensor? = null
    private var compassAccel: FloatArray? = null
    private var compassMag: FloatArray? = null
    private var lastCompassRadians: Float? = null
    private var lastCompassDispatchTs: Long = 0L
    private var compassActive = false
    private var compassUnavailable = false

    private var barometerThread: HandlerThread? = null
    private var barometerHandler: Handler? = null
    private var barometerListener: SensorEventListener? = null
    private var barometerSensor: Sensor? = null
    private var lastBarometerDispatch: Long = 0L

    private var magnetometerThread: HandlerThread? = null
    private var magnetometerHandler: Handler? = null
    private var magnetometerListener: SensorEventListener? = null
    private var magnetometerSensor: Sensor? = null
    private var lastMagnetometerDispatch: Long = 0L

    fun startLogging(bindings: Map<String, String>) {
        val mgr = activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager ?: return
        val accel = mgr.getDefaultSensor(Sensor.TYPE_ACCELEROMETER)
        val gyro = mgr.getDefaultSensor(Sensor.TYPE_GYROSCOPE)
        val mag = mgr.getDefaultSensor(Sensor.TYPE_MAGNETIC_FIELD)
        val pressure = mgr.getDefaultSensor(Sensor.TYPE_PRESSURE)

        val config = parseSensorConfig(bindings)
        if (config == null) {
            refreshUi("sensor_logger_status", mapOf("sensor_status" to "invalid_config"))
            return
        }

        val thread = HandlerThread("SensorLogger")
        thread.start()
        sensorThread = thread
        sensorHandler = Handler(thread.looper)

        val dir = activity.getExternalFilesDir(null) ?: activity.filesDir
        logFile = File(dir, "sensors_${System.currentTimeMillis()}.csv")
        logWriter = runCatching { OutputStreamWriter(FileOutputStream(logFile!!)) }.getOrNull()
        if (logWriter == null) {
            refreshUi("sensor_logger_status", mapOf("sensor_status" to "log_open_failed"))
            return
        }
        logWriter?.write("ts,sensor,x,y,z,extra1,extra2\n")

        val listener = object : SensorEventListener {
            override fun onSensorChanged(event: SensorEvent) {
                if (!isLogging) return
                val ts = System.currentTimeMillis()
                val values = event.values
                val writer = logWriter ?: return
                val bindings = mutableMapOf<String, String>()
                bindings["sensor_status"] = "logging"
                logFile?.absolutePath?.let { bindings["sensor_path"] = it }
                try {
                    when (event.sensor.type) {
                        Sensor.TYPE_ACCELEROMETER -> {
                            if (!config.selection.accel) return
                            writer.write(formatRow(ts, "ACCEL", values))
                        }
                        Sensor.TYPE_GYROSCOPE -> {
                            if (!config.selection.gyro) return
                            writer.write(formatRow(ts, "GYRO", values))
                        }
                        Sensor.TYPE_MAGNETIC_FIELD -> {
                            if (!config.selection.mag) return
                            writer.write(formatRow(ts, "MAG", values))
                        }
                        Sensor.TYPE_PRESSURE -> {
                            if (!config.selection.pressure) return
                            writer.write("${ts},BARO,${values.getOrNull(0) ?: 0f},0,0,0,0\n")
                        }
                    }
                    writer.flush()
                    lastSensorLogPath = logFile?.absolutePath
                    val nowMono = android.os.SystemClock.elapsedRealtime()
                    if (nowMono - lastSensorUiTs > 500) {
                        lastSensorUiTs = nowMono
                        refreshUi("sensor_logger_status", bindings)
                    }
                } catch (_: Exception) {
                }
            }

            override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit
        }
        sensorListener = listener

        if (config.selection.accel) accel?.let { mgr.registerListener(listener, it, config.intervalMs.toInt(), sensorHandler) }
        if (config.selection.gyro) gyro?.let { mgr.registerListener(listener, it, config.intervalMs.toInt(), sensorHandler) }
        if (config.selection.mag) mag?.let { mgr.registerListener(listener, it, config.intervalMs.toInt(), sensorHandler) }
        if (config.selection.pressure) pressure?.let { mgr.registerListener(listener, it, config.intervalMs.toInt(), sensorHandler) }

        isLogging = true

        if (config.selection.gps) {
            if (!hasLocationPermission()) {
                pendingSensorStart = true
                pendingSensorBindings = bindings
                activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION), MainActivity.PERMISSION_LOCATION)
            } else {
                startLocationLogging(config)
            }
        }
    }

    fun stopLogging() {
        isLogging = false
        (activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager)?.unregisterListener(sensorListener)
        sensorListener = null
        sensorHandler = null
        sensorThread?.quitSafely()
        sensorThread = null
        stopLocationLogging()
        runCatching { logWriter?.close() }
        logWriter = null
        val bindings = mutableMapOf<String, String>()
        bindings["sensor_status"] = "stopped"
        lastSensorLogPath?.let { bindings["sensor_path"] = it }
        refreshUi("sensor_logger_status", bindings)
    }

    fun shareLastLog() {
        val path = lastSensorLogPath ?: return
        val file = File(path)
        if (!file.exists()) return
        val uri = androidx.core.content.FileProvider.getUriForFile(
            activity,
            "${activity.packageName}.fileprovider",
            file
        )
        val intent = Intent(Intent.ACTION_SEND).apply {
            type = "text/csv"
            putExtra(Intent.EXTRA_STREAM, uri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        activity.startActivity(Intent.createChooser(intent, "Share sensor log"))
    }

    fun onPermissionResult(requestCode: Int, grantResults: IntArray, bindings: Map<String, String>) {
        if (requestCode != MainActivity.PERMISSION_LOCATION) return
        val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
        if (granted && pendingSensorStart) {
            pendingSensorStart = false
            val pending = pendingSensorBindings ?: bindings
            pendingSensorBindings = null
            startLogging(pending)
        } else {
            pendingSensorBindings = null
            refreshUi("sensor_logger_status", mapOf("sensor_status" to "location permission denied"))
        }
    }

    fun startCompass() {
        if (compassActive) return
        compassUnavailable = false
        if (compassSensorManager == null) {
            compassSensorManager = activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager
        }
        val mgr = compassSensorManager ?: return
        val rotationSensor = mgr.getDefaultSensor(Sensor.TYPE_ROTATION_VECTOR)
        val accelSensor = mgr.getDefaultSensor(Sensor.TYPE_ACCELEROMETER)
        val magSensor = mgr.getDefaultSensor(Sensor.TYPE_MAGNETIC_FIELD)
        if (rotationSensor == null && (accelSensor == null || magSensor == null)) {
            if (!compassUnavailable) {
                compassUnavailable = true
                sendSensorUpdate("compass_set", extras = mapOf("angle_radians" to 0.0, "error" to "Compass sensors unavailable"))
            }
            return
        }
        val thread = HandlerThread("CompassListener")
        thread.start()
        compassThread = thread
        compassHandler = Handler(thread.looper)
        compassRotationSensor = rotationSensor
        compassListener = object : SensorEventListener {
            override fun onSensorChanged(event: SensorEvent) {
                when (event.sensor.type) {
                    Sensor.TYPE_ROTATION_VECTOR -> {
                        val rotMatrix = FloatArray(9)
                        SensorManager.getRotationMatrixFromVector(rotMatrix, event.values)
                        val orientation = FloatArray(3)
                        SensorManager.getOrientation(rotMatrix, orientation)
                        notifyCompassAngle(orientation[0])
                    }
                    Sensor.TYPE_ACCELEROMETER -> {
                        compassAccel = event.values.clone()
                        maybeComputeAccelMag()
                    }
                    Sensor.TYPE_MAGNETIC_FIELD -> {
                        compassMag = event.values.clone()
                        maybeComputeAccelMag()
                    }
                }
            }

            override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit

            private fun maybeComputeAccelMag() {
                val accel = compassAccel
                val mag = compassMag
                if (accel != null && mag != null) {
                    val r = FloatArray(9)
                    val i = FloatArray(9)
                    if (SensorManager.getRotationMatrix(r, i, accel, mag)) {
                        val orientation = FloatArray(3)
                        SensorManager.getOrientation(r, orientation)
                        notifyCompassAngle(orientation[0])
                    }
                }
            }
        }
        compassListener?.let { listener ->
            if (rotationSensor != null) {
                mgr.registerListener(listener, rotationSensor, SensorManager.SENSOR_DELAY_UI, compassHandler)
            } else {
                accelSensor?.let { mgr.registerListener(listener, it, SensorManager.SENSOR_DELAY_UI, compassHandler) }
                magSensor?.let { mgr.registerListener(listener, it, SensorManager.SENSOR_DELAY_UI, compassHandler) }
            }
        }
        compassActive = true
    }

    fun stopCompass() {
        if (!compassActive) return
        compassActive = false
        compassUnavailable = false
        val mgr = compassSensorManager
        val listener = compassListener
        if (mgr != null && listener != null) {
            mgr.unregisterListener(listener)
        }
        compassListener = null
        compassAccel = null
        compassMag = null
        compassRotationSensor = null
        compassHandler = null
        compassThread?.quitSafely()
        compassThread = null
    }

    fun startBarometer() {
        if (barometerListener != null) return
        if (barometerSensor == null) {
            barometerSensor = (activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager)
                ?.getDefaultSensor(Sensor.TYPE_PRESSURE)
        }
        val sensor = barometerSensor ?: run {
            sendSensorUpdate("barometer_set", extras = mapOf("angle_radians" to 0.0, "error" to "Barometer unavailable"))
            return
        }
        val thread = HandlerThread("BarometerListener")
        thread.start()
        barometerThread = thread
        barometerHandler = Handler(thread.looper)
        barometerListener = object : SensorEventListener {
            override fun onSensorChanged(event: SensorEvent) {
                val hpa = event.values.firstOrNull()?.toDouble() ?: return
                val now = android.os.SystemClock.elapsedRealtime()
                if (now - lastBarometerDispatch < 300) return
                lastBarometerDispatch = now
                sendSensorUpdate("barometer_set", extras = mapOf("angle_radians" to hpa, "error" to JSONObject.NULL))
            }

            override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit
        }
        barometerListener?.let { listener ->
            (activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager)
                ?.registerListener(listener, sensor, SensorManager.SENSOR_DELAY_UI, barometerHandler)
        }
    }

    fun stopBarometer() {
        val mgr = activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager
        barometerListener?.let { mgr?.unregisterListener(it) }
        barometerListener = null
        barometerHandler = null
        barometerThread?.quitSafely()
        barometerThread = null
    }

    fun startMagnetometer() {
        if (magnetometerListener != null) return
        if (magnetometerSensor == null) {
            magnetometerSensor = (activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager)
                ?.getDefaultSensor(Sensor.TYPE_MAGNETIC_FIELD)
        }
        val sensor = magnetometerSensor ?: run {
            sendSensorUpdate("magnetometer_set", extras = mapOf("angle_radians" to 0.0, "error" to "Magnetometer unavailable"))
            return
        }
        val thread = HandlerThread("MagnetometerListener")
        thread.start()
        magnetometerThread = thread
        magnetometerHandler = Handler(thread.looper)
        magnetometerListener = object : SensorEventListener {
            override fun onSensorChanged(event: SensorEvent) {
                val vals = event.values
                if (vals.size < 3) return
                val mag = kotlin.math.sqrt((vals[0] * vals[0] + vals[1] * vals[1] + vals[2] * vals[2]).toDouble())
                val now = android.os.SystemClock.elapsedRealtime()
                if (now - lastMagnetometerDispatch < 300) return
                lastMagnetometerDispatch = now
                sendSensorUpdate("magnetometer_set", extras = mapOf("angle_radians" to mag, "error" to JSONObject.NULL))
            }

            override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit
        }
        magnetometerListener?.let { listener ->
            (activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager)
                ?.registerListener(listener, sensor, SensorManager.SENSOR_DELAY_UI, magnetometerHandler)
        }
    }

    fun stopMagnetometer() {
        val mgr = activity.getSystemService(Context.SENSOR_SERVICE) as? SensorManager
        magnetometerListener?.let { mgr?.unregisterListener(it) }
        magnetometerListener = null
        magnetometerHandler = null
        magnetometerThread?.quitSafely()
        magnetometerThread = null
    }

    fun updateSubscriptions(json: String) {
        val wantsCompass = jsonHasWidget(json, "Compass")
        if (wantsCompass) startCompass() else stopCompass()

        val wantsBaro = jsonHasWidget(json, "Barometer")
        if (wantsBaro) startBarometer() else stopBarometer()

        val wantsMag = jsonHasWidget(json, "Magnetometer")
        if (wantsMag) startMagnetometer() else stopMagnetometer()
    }

    fun onDestroy() {
        pendingSensorStart = false
        pendingSensorBindings = null
        stopLogging()
        stopCompass()
        stopBarometer()
        stopMagnetometer()
    }

    fun lastLogPath(): String? = lastSensorLogPath

    private fun parseSensorConfig(bindings: Map<String, String>): SensorConfig? {
        val selection = SensorSelection(
            accel = bindings["sensor_accel"]?.toBoolean() ?: true,
            gyro = bindings["sensor_gyro"]?.toBoolean() ?: true,
            mag = bindings["sensor_mag"]?.toBoolean() ?: true,
            pressure = bindings["sensor_pressure"]?.toBoolean() ?: false,
            gps = bindings["sensor_gps"]?.toBoolean() ?: false,
            battery = bindings["sensor_battery"]?.toBoolean() ?: true,
        )
        if (!selection.any()) return null
        val interval = bindings["sensor_interval_ms"]?.toLongOrNull()?.coerceIn(50, 10_000) ?: 200
        return SensorConfig(selection, interval)
    }

    private fun formatRow(ts: Long, name: String, vals: FloatArray): String {
        val x = vals.getOrNull(0) ?: 0f
        val y = vals.getOrNull(1) ?: 0f
        val z = vals.getOrNull(2) ?: 0f
        return String.format(Locale.US, "%d,%s,%.5f,%.5f,%.5f,0,0\n", ts, name, x, y, z)
    }

    private fun hasLocationPermission(): Boolean {
        return ContextCompat.checkSelfPermission(activity, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED
    }

    private fun startLocationLogging(config: SensorConfig) {
        if (locationManager == null) {
            locationManager = activity.getSystemService(Context.LOCATION_SERVICE) as? LocationManager
        }
        val mgr = locationManager ?: return
        val listener = object : LocationListener {
            override fun onLocationChanged(location: Location) {
                if (!isLogging) return
                val ts = System.currentTimeMillis()
                val nowMono = android.os.SystemClock.elapsedRealtime()
                val lat = location.latitude
                val lon = location.longitude
                val acc = location.accuracy.toDouble()
                val writer = logWriter ?: return
                val bindings = mutableMapOf<String, String>()
                bindings["sensor_status"] = "logging"
                logFile?.absolutePath?.let { bindings["sensor_path"] = it }
                try {
                    writer.write("$ts,GPS,$lat,$lon,$acc,-1,-1\n")
                    writer.flush()
                    lastSensorLogPath = logFile?.absolutePath
                    if (nowMono - lastSensorUiTs > 500) {
                        lastSensorUiTs = nowMono
                        refreshUi("sensor_logger_status", bindings)
                    }
                } catch (_: Exception) {
                }
            }

            override fun onStatusChanged(provider: String?, status: Int, extras: android.os.Bundle?) = Unit
            override fun onProviderEnabled(provider: String) = Unit
            override fun onProviderDisabled(provider: String) = Unit
        }
        locationListener = listener
        try {
            val interval = config.intervalMs.coerceIn(1000, 10_000)
            mgr.requestLocationUpdates(LocationManager.GPS_PROVIDER, interval, 0f, listener, sensorHandler?.looper)
        } catch (_: SecurityException) {
            // permission denied, status already handled elsewhere
            Toast.makeText(activity, "Location permission denied", Toast.LENGTH_SHORT).show()
        }
    }

    private fun stopLocationLogging() {
        locationManager?.removeUpdates(locationListener ?: return)
        locationListener = null
    }

    private fun notifyCompassAngle(raw: Float) {
        val now = android.os.SystemClock.elapsedRealtime()
        val tau = (2 * Math.PI).toFloat()
        val normalized = ((raw % tau) + tau) % tau
        val prev = lastCompassRadians
        val minDelta = Math.toRadians(1.0).toFloat()
        val minIntervalMs = 300L
        if (prev != null && kotlin.math.abs(prev - normalized) < minDelta && now - lastCompassDispatchTs < minIntervalMs) {
            return
        }
        lastCompassRadians = normalized
        sendSensorUpdate(
            "compass_set",
            extras = mapOf("angle_radians" to normalized.toDouble(), "error" to JSONObject.NULL)
        )
        lastCompassDispatchTs = now
    }

    private fun jsonHasWidget(json: String, widgetType: String): Boolean {
        val root = runCatching { JSONObject(json) }.getOrNull() ?: return false
        fun walk(obj: JSONObject): Boolean {
            if (obj.optString("type") == widgetType) return true
            val children = obj.optJSONArray("children") ?: return false
            for (i in 0 until children.length()) {
                val child = children.optJSONObject(i) ?: continue
                if (walk(child)) return true
            }
            return false
        }
        return walk(root)
    }

    private fun sendSensorUpdate(action: String, extras: Map<String, Any?>) {
        scope.launch {
            launch(Dispatchers.IO) {
                val command = JSONObject().apply {
                    put("action", action)
                    extras.forEach { (k, v) -> put(k, v) }
                }
                dispatchRaw(command.toString())
            }
        }
    }

    data class SensorSelection(
        val accel: Boolean,
        val gyro: Boolean,
        val mag: Boolean,
        val pressure: Boolean,
        val gps: Boolean,
        val battery: Boolean,
    ) {
        fun any(): Boolean = accel || gyro || mag || pressure || gps || battery
    }

    data class SensorConfig(
        val selection: SensorSelection,
        val intervalMs: Long
    )
}
