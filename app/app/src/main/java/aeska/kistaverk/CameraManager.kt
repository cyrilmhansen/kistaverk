package aeska.kistaverk

import android.Manifest
import android.content.pm.PackageManager
import android.widget.FrameLayout
import android.widget.Toast
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.content.ContextCompat
import androidx.lifecycle.LifecycleOwner

/**
 * QR camera lifecycle helper. Manages CameraX use cases and feeds frames to JNI.
 */
class CameraManager(
    private val activity: MainActivity,
    private val processFrame: (ByteArray, Int, Int, Int, Int) -> String?,
    private val dispatchAction: (String, Map<String, String>) -> Unit
) {
    private var cameraProvider: ProcessCameraProvider? = null
    private var imageAnalyzer: ImageAnalysis? = null
    private var previewView: PreviewView? = null
    private var isQrScanActive = false
    private var hasCameraPermission = false

    fun ensurePreview(container: FrameLayout) {
        if (previewView != null) return
        previewView = PreviewView(activity).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        }
        container.addView(previewView, 0)
    }

    fun onScreenChanged(isQrScreen: Boolean, container: FrameLayout?) {
        if (isQrScreen) {
            container?.let { ensurePreview(it) }
            startQrScanner()
        } else {
            stopQrScanner()
        }
    }

    fun startQrScanner() {
        val granted = ContextCompat.checkSelfPermission(activity, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
        hasCameraPermission = granted
        if (granted) {
            startCameraX()
        } else {
            Toast.makeText(activity, "Camera permission required for QR scanning", Toast.LENGTH_SHORT).show()
            activity.requestPermissions(arrayOf(Manifest.permission.CAMERA), MainActivity.CAMERA_PERMISSION_REQUEST_CODE)
        }
    }

    fun onPermissionResult(grantResults: IntArray) {
        if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
            hasCameraPermission = true
            startQrScanner()
        } else {
            hasCameraPermission = false
            Toast.makeText(activity, "Camera permission denied", Toast.LENGTH_SHORT).show()
            activity.refreshUi("qr_receive_screen", extras = mapOf("error" to "camera_permission_denied"))
        }
    }

    fun stopQrScanner() {
        isQrScanActive = false
        cameraProvider?.unbindAll()
        imageAnalyzer?.clearAnalyzer()
        cameraProvider = null
        imageAnalyzer = null
        previewView = null
    }

    private fun startCameraX() {
        if (isQrScanActive) return
        isQrScanActive = true

        val cameraProviderFuture = ProcessCameraProvider.getInstance(activity)
        cameraProviderFuture.addListener({
            cameraProvider = cameraProviderFuture.get()
            bindCameraUseCases()
        }, ContextCompat.getMainExecutor(activity))
    }

    private fun bindCameraUseCases() {
        val provider = cameraProvider ?: return
        val previewView = previewView ?: return

        provider.unbindAll()

        val cameraSelector = CameraSelector.Builder()
            .requireLensFacing(CameraSelector.LENS_FACING_BACK)
            .build()

        val preview = Preview.Builder().build().also {
            it.setSurfaceProvider(previewView.surfaceProvider)
        }

        imageAnalyzer = ImageAnalysis.Builder()
            .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
            .build()
            .also {
                it.setAnalyzer(ContextCompat.getMainExecutor(activity), QrCodeAnalyzer(processFrame) { qrResult ->
                    if (qrResult != null) {
                        isQrScanActive = false // Stop further scanning
                        provider.unbindAll()
                        dispatchAction(
                            "qr_receive_scan",
                            mapOf("qr_scan_input" to qrResult)
                        )
                    }
                })
            }

        try {
            provider.bindToLifecycle(activity as LifecycleOwner, cameraSelector, preview, imageAnalyzer)
        } catch (exc: Exception) {
            isQrScanActive = false
            activity.refreshUi("qr_receive_screen", extras = mapOf("error" to "camera_bind_failed"))
        }
    }

    private class QrCodeAnalyzer(
        private val jniProcessor: (ByteArray, Int, Int, Int, Int) -> String?,
        private val listener: (String?) -> Unit
    ) : ImageAnalysis.Analyzer {
        private var lastAnalyzedTimestamp = 0L

        override fun analyze(image: ImageProxy) {
            val currentTimestamp = System.currentTimeMillis()
            if (currentTimestamp - lastAnalyzedTimestamp < 100) { // Throttle analysis
                image.close()
                return
            }
            lastAnalyzedTimestamp = currentTimestamp

            val yBuffer = image.planes[0].buffer
            val yBytes = ByteArray(yBuffer.remaining())
            yBuffer.get(yBytes)

            val width = image.width
            val height = image.height
            val rowStride = image.planes[0].rowStride
            val rotationDeg = image.imageInfo.rotationDegrees

            val result = jniProcessor(yBytes, width, height, rowStride, rotationDeg)

            image.close()
            if (result != null) {
                listener(result)
            }
        }
    }
}
