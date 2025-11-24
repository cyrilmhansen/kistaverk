package aeska.kistaverk.features

import android.content.Context
import android.content.ContentValues
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import androidx.documentfile.provider.DocumentFile
import java.io.File
import java.io.FileOutputStream
import java.text.DecimalFormat

object KotlinImageConversion {
    fun isConversionAction(action: String): Boolean {
        return action == "kotlin_image_convert_webp" || action == "kotlin_image_convert_png"
    }

    fun convert(
        context: Context,
        cacheDir: File,
        picturesDir: File?,
        outputDirUri: Uri?,
        uri: Uri,
        action: String
    ): ConversionResult {
        val target = targetForAction(action)
            ?: return ConversionResult.Failure(target = null, reason = "unsupported_action")

        val resolver = context.contentResolver

        return runCatching {
            val bitmap = resolver.openInputStream(uri)?.use { input ->
                BitmapFactory.decodeStream(input)
            } ?: error("decode_failed")

            val result = if (outputDirUri != null) {
                val tree = DocumentFile.fromTreeUri(context, outputDirUri) ?: error("open_dir_failed")
                val outDoc = tree.createFile(target.mimeType, outputName(target)) ?: error("create_failed")
                resolver.openOutputStream(outDoc.uri)?.use { out ->
                    if (!bitmap.compress(target.format, target.quality, out)) {
                        error("compress_failed")
                    }
                } ?: error("open_output_failed")
                val size = outDoc.length().takeIf { it > 0 } ?: resolver.openAssetFileDescriptor(outDoc.uri, "r")?.use { it.length } ?: 0L
                ConversionResult.Success(
                    destination = outDoc.uri.toString(),
                    format = target.extension.uppercase(),
                    size = readableBytes(size),
                    target = target
                )
            } else {
                mediaStoreSave(context, bitmap, target)
                    ?: fileSave(cacheDir, picturesDir, bitmap, target)
            }
            bitmap.recycle()
            result
        }.getOrElse { throwable ->
            ConversionResult.Failure(
                target = target,
                reason = throwable.message ?: "conversion_failed"
            )
        }
    }

    private fun mediaStoreSave(
        context: Context,
        bitmap: Bitmap,
        target: ImageTarget
    ): ConversionResult.Success? {
        val resolver = context.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, outputName(target))
            put(MediaStore.Images.Media.MIME_TYPE, target.mimeType)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.Images.Media.RELATIVE_PATH, "Pictures/kistaverk")
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }

        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: return null
        return runCatching {
            resolver.openOutputStream(uri)?.use { out ->
                if (!bitmap.compress(target.format, target.quality, out)) {
                    error("compress_failed")
                }
            } ?: error("open_output_failed")

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val done = ContentValues().apply {
                    put(MediaStore.Images.Media.IS_PENDING, 0)
                }
                resolver.update(uri, done, null, null)
            }

            val size = resolver.openAssetFileDescriptor(uri, "r")?.use { it.length } ?: 0L
            ConversionResult.Success(
                destination = uri.toString(),
                format = target.extension.uppercase(),
                size = readableBytes(size),
                target = target
            )
        }.getOrElse {
            resolver.delete(uri, null, null)
            null
        }
    }

    private fun fileSave(
        cacheDir: File,
        picturesDir: File?,
        bitmap: Bitmap,
        target: ImageTarget
    ): ConversionResult.Success {
        val baseDir = ensureOutputDir(cacheDir, picturesDir)
        val outFile = File(baseDir, outputName(target))
        FileOutputStream(outFile).use { out ->
            if (!bitmap.compress(target.format, target.quality, out)) {
                error("compress_failed")
            }
        }
        return ConversionResult.Success(
            destination = outFile.absolutePath,
            format = target.extension.uppercase(),
            size = readableBytes(outFile.length()),
            target = target
        )
    }

    private fun ensureOutputDir(cacheDir: File, picturesDir: File?): File {
        val preferred = picturesDir?.let { File(it, "kistaverk") }
        if (preferred != null) {
            preferred.mkdirs()
            if (preferred.exists() && preferred.isDirectory) {
                return preferred
            }
        }
        val fallback = File(cacheDir, "kistaverk")
        fallback.mkdirs()
        return fallback
    }

    private fun outputName(target: ImageTarget): String {
        return "converted_${System.currentTimeMillis()}.${target.extension}"
    }

    private fun targetForAction(action: String): ImageTarget? {
        return when (action) {
            "kotlin_image_convert_webp" -> ImageTarget("webp", Bitmap.CompressFormat.WEBP_LOSSLESS, "webp", 100, "image/webp")
            "kotlin_image_convert_png" -> ImageTarget("png", Bitmap.CompressFormat.PNG, "png", 100, "image/png")
            else -> null
        }
    }

    private fun readableBytes(bytes: Long): String {
        if (bytes <= 0) return "0 B"
        val units = arrayOf("B", "KB", "MB", "GB")
        val group = (Math.log10(bytes.toDouble()) / Math.log10(1024.0)).toInt().coerceIn(0, units.lastIndex)
        val formatter = DecimalFormat("#.#")
        val value = bytes / Math.pow(1024.0, group.toDouble())
        return "${formatter.format(value)} ${units[group]}"
    }
}

data class ImageTarget(
    val key: String,
    val format: Bitmap.CompressFormat,
    val extension: String,
    val quality: Int,
    val mimeType: String
)

sealed class ConversionResult {
    data class Success(
        val destination: String,
        val format: String,
        val size: String,
        val target: ImageTarget
    ) : ConversionResult()

    data class Failure(
        val target: ImageTarget?,
        val reason: String?
    ) : ConversionResult()
}
