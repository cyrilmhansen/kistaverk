package aeska.kistaverk.features

import android.content.Context
import android.content.ContentValues
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import androidx.documentfile.provider.DocumentFile
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.text.DecimalFormat
import java.util.Locale

object KotlinImageConversion {
    data class ResizeOptions(
        val scalePercent: Int,
        val quality: Int,
        val targetBytes: Long?,
        val target: ImageTarget
    )

    private data class Compressed(
        val bytes: ByteArray,
        val quality: Int
    )

    fun isConversionAction(action: String): Boolean {
        return action == "kotlin_image_convert_webp" ||
            action == "kotlin_image_convert_png" ||
            action == "kotlin_image_resize"
    }

    fun convert(
        context: Context,
        cacheDir: File,
        picturesDir: File?,
        outputDirUri: Uri?,
        uri: Uri,
        action: String,
        bindings: Map<String, String> = emptyMap()
    ): ConversionResult {
        val target = targetForAction(action)
            ?: return ConversionResult.Failure(target = null, reason = "unsupported_action")

        val resolver = context.contentResolver

        return runCatching {
            val bitmap = resolver.openInputStream(uri)?.use { input ->
                BitmapFactory.decodeStream(input)
            } ?: error("decode_failed")

            val compressed = compressToBytes(bitmap, target, target.quality)
            val result = saveBytes(
                context = context,
                cacheDir = cacheDir,
                picturesDir = picturesDir,
                outputDirUri = outputDirUri,
                compressed = compressed,
                target = target,
                prefix = "converted",
                scalePercent = null,
                targetBytes = null
            )
            bitmap.recycle()
            result
        }.getOrElse { throwable ->
            ConversionResult.Failure(
                target = target,
                reason = throwable.message ?: "conversion_failed"
            )
        }
    }

    fun resize(
        context: Context,
        cacheDir: File,
        picturesDir: File?,
        outputDirUri: Uri?,
        uri: Uri,
        bindings: Map<String, String>
    ): ConversionResult {
        val opts = buildResizeOptions(bindings)
        val resolver = context.contentResolver
        return runCatching {
            val bitmap = resolver.openInputStream(uri)?.use { input ->
                BitmapFactory.decodeStream(input)
            } ?: error("decode_failed")

            val resized = scaleBitmap(bitmap, opts.scalePercent)
            val compressed = compressWithBudget(resized, opts.target, opts.quality, opts.targetBytes)
            val result = saveBytes(
                context = context,
                cacheDir = cacheDir,
                picturesDir = picturesDir,
                outputDirUri = outputDirUri,
                compressed = compressed,
                target = opts.target.copy(quality = compressed.quality),
                prefix = "resized",
                scalePercent = opts.scalePercent,
                targetBytes = opts.targetBytes
            )
            if (resized !== bitmap) {
                bitmap.recycle()
            }
            resized.recycle()
            result
        }.getOrElse { throwable ->
            ConversionResult.Failure(
                target = opts.target,
                reason = throwable.message ?: "resize_failed"
            )
        }
    }

    private fun compressWithBudget(
        bitmap: Bitmap,
        target: ImageTarget,
        quality: Int,
        targetBytes: Long?
    ): Compressed {
        var q = quality.coerceIn(40, 100)
        var compressed = compressToBytes(bitmap, target, q)
        if (targetBytes != null && target.format != Bitmap.CompressFormat.PNG) {
            var attempts = 0
            while (compressed.bytes.size.toLong() > targetBytes && attempts < 5 && q > 40) {
                q = (q - 10).coerceAtLeast(40)
                compressed = compressToBytes(bitmap, target, q)
                attempts += 1
            }
        }
        return compressed
    }

    private fun compressToBytes(bitmap: Bitmap, target: ImageTarget, quality: Int): Compressed {
        val stream = ByteArrayOutputStream()
        if (!bitmap.compress(target.format, quality, stream)) {
            error("compress_failed")
        }
        return Compressed(stream.toByteArray(), quality)
    }

    private fun saveBytes(
        context: Context,
        cacheDir: File,
        picturesDir: File?,
        outputDirUri: Uri?,
        compressed: Compressed,
        target: ImageTarget,
        prefix: String,
        scalePercent: Int?,
        targetBytes: Long?
    ): ConversionResult.Success {
        val resolver = context.contentResolver
        val displayName = ensureExtension(outputName(target, prefix), target.extension)

        if (outputDirUri != null) {
            val tree = DocumentFile.fromTreeUri(context, outputDirUri)
            if (tree != null) {
                val outDoc = tree.createFile(target.mimeType, displayName)
                if (outDoc != null) {
                    resolver.openOutputStream(outDoc.uri)?.use { out ->
                        out.write(compressed.bytes)
                    } ?: error("open_output_failed")
                    val size = compressed.bytes.size.toLong()
                    return ConversionResult.Success(
                        destination = outDoc.uri.toString(),
                        format = target.extension.uppercase(),
                        size = readableBytes(size),
                        target = target,
                        quality = compressed.quality,
                        scalePercent = scalePercent,
                        targetBytes = targetBytes
                    )
                }
            }
        }

        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, displayName)
            put(MediaStore.Images.Media.MIME_TYPE, target.mimeType)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.Images.Media.RELATIVE_PATH, "Pictures/kistaverk")
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }

        val insertedUri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
        if (insertedUri != null) {
            val success = runCatching {
                resolver.openOutputStream(insertedUri)?.use { out ->
                    out.write(compressed.bytes)
                } ?: error("open_output_failed")

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    val done = ContentValues().apply { put(MediaStore.Images.Media.IS_PENDING, 0) }
                    resolver.update(insertedUri, done, null, null)
                }
                ConversionResult.Success(
                    destination = insertedUri.toString(),
                    format = target.extension.uppercase(),
                    size = readableBytes(compressed.bytes.size.toLong()),
                    target = target,
                    quality = compressed.quality,
                    scalePercent = scalePercent,
                    targetBytes = targetBytes
                )
            }.getOrNull()
            if (success != null) {
                return success
            } else {
                resolver.delete(insertedUri, null, null)
            }
        }

        val baseDir = ensureOutputDir(cacheDir, picturesDir)
        val outFile = File(baseDir, displayName)
        FileOutputStream(outFile).use { out -> out.write(compressed.bytes) }
        return ConversionResult.Success(
            destination = outFile.absolutePath,
            format = target.extension.uppercase(),
            size = readableBytes(outFile.length()),
            target = target,
            quality = compressed.quality,
            scalePercent = scalePercent,
            targetBytes = targetBytes
        )
    }

    private fun scaleBitmap(source: Bitmap, scalePercent: Int): Bitmap {
        val pct = scalePercent.coerceIn(5, 100)
        if (pct >= 100) return source
        val width = (source.width * pct / 100f).toInt().coerceAtLeast(1)
        val height = (source.height * pct / 100f).toInt().coerceAtLeast(1)
        return Bitmap.createScaledBitmap(source, width, height, true)
    }

    private fun buildResizeOptions(bindings: Map<String, String>): ResizeOptions {
        val scale = bindings["resize_scale_pct"]?.toIntOrNull()?.coerceIn(5, 100) ?: 70
        val quality = bindings["resize_quality"]?.toIntOrNull()?.coerceIn(40, 100) ?: 85
        val targetBytes = bindings["resize_target_kb"]?.toLongOrNull()?.takeIf { it > 0 }
            ?.coerceAtMost(10_000)
            ?.times(1024)
        val useWebp = bindings["resize_use_webp"]?.toBooleanStrictOrNull() ?: false
        val target = if (useWebp) webpTarget(quality) else jpegTarget(quality)
        return ResizeOptions(
            scalePercent = scale,
            quality = quality,
            targetBytes = targetBytes,
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

    private fun outputName(target: ImageTarget, prefix: String): String {
        return "${prefix}_${System.currentTimeMillis()}.${target.extension}"
    }

    private fun ensureExtension(name: String, extension: String): String {
        val lower = name.lowercase(Locale.US)
        val suffix = ".${extension.lowercase(Locale.US)}"
        return if (lower.endsWith(suffix)) name else "$name$suffix"
    }

    private fun targetForAction(action: String): ImageTarget? {
        return when (action) {
            "kotlin_image_convert_webp" -> webpTarget(100)
            "kotlin_image_convert_png" -> pngTarget()
            else -> null
        }
    }

    private fun webpTarget(quality: Int): ImageTarget {
        return ImageTarget("webp", webpFormat(), "webp", quality, "image/webp")
    }

    private fun webpFormat(): Bitmap.CompressFormat {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Bitmap.CompressFormat.WEBP_LOSSY
        } else {
            Bitmap.CompressFormat.WEBP
        }
    }

    private fun jpegTarget(quality: Int): ImageTarget {
        return ImageTarget("jpeg", Bitmap.CompressFormat.JPEG, "jpg", quality, "image/jpeg")
    }

    private fun pngTarget(): ImageTarget {
        return ImageTarget("png", Bitmap.CompressFormat.PNG, "png", 100, "image/png")
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
        val target: ImageTarget,
        val quality: Int? = null,
        val scalePercent: Int? = null,
        val targetBytes: Long? = null
    ) : ConversionResult()

    data class Failure(
        val target: ImageTarget?,
        val reason: String?
    ) : ConversionResult()
}
