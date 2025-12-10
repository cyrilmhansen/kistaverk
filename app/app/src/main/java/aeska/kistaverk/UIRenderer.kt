package aeska.kistaverk

import android.content.Context
import android.graphics.Color
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.drawable.GradientDrawable
import android.graphics.pdf.PdfRenderer
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.content.ClipData
import android.content.ClipboardManager
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.os.Handler
import android.os.Looper
import android.util.Base64
import android.view.View
import android.view.MotionEvent
import android.view.ViewGroup
import android.view.Gravity
import android.graphics.Matrix
import android.view.inputmethod.EditorInfo
import android.text.method.PasswordTransformationMethod
import android.text.InputType
import android.widget.Button
import android.widget.EditText
import android.widget.CheckBox
import android.widget.GridLayout
import android.widget.LinearLayout.LayoutParams
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.Space
import android.webkit.WebSettings
import android.webkit.WebView
import android.widget.ScrollView
import android.widget.ImageView
import android.widget.TextView
import java.io.ByteArrayOutputStream
import org.json.JSONArray
import android.text.Editable
import android.text.TextWatcher
import android.widget.ProgressBar
import android.widget.HorizontalScrollView
import org.json.JSONObject
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.cos
import kotlin.math.sin

// Added 'onAction' callback: (String, Boolean) -> Unit where the boolean flags file picker needs
class UiRenderer(
    private val context: Context,
    private val onAction: (String, Boolean, Boolean, Map<String, String>) -> Unit
) {
    private data class RenderMeta(val type: String, val nodeId: String?)
    private data class PdfPickerCache(val uri: String, val pageCount: Int)
    private data class SignatureState(val base64: String, val widthPx: Int, val heightPx: Int, val dpi: Float)

    private val renderMetaTag = R.id.render_meta_tag
    private val bindKeyTag = R.id.bind_key_tag
    private val dataTag = R.id.data_tag
    private val creators: Map<String, (JSONObject, View?) -> View> = mapOf(
        "Column" to { data, matched -> createColumn(data, matched as? LinearLayout) },
        "Section" to { data, matched -> createSection(data, matched as? LinearLayout) },
        "Card" to { data, matched -> createCard(data, matched as? LinearLayout) },
        "Text" to { data, matched -> createText(data, matched as? TextView) },
        "Button" to { data, matched -> createButton(data, matched as? Button) },
        "ShaderToy" to { data, matched -> createShaderToy(data, matched as? ShaderToyView) },
        "TextInput" to { data, matched -> createTextInput(data, matched as? EditText) },
        "Checkbox" to { data, matched -> createCheckbox(data, matched as? CheckBox) },
        "Progress" to { data, matched -> createProgress(data, matched as? LinearLayout) },
        "Grid" to { data, matched -> createGrid(data, matched as? LinearLayout) },
        "VirtualList" to { data, matched -> createVirtualList(data, matched as? LinearLayout) },
        "ImageBase64" to { data, matched -> createImageBase64(data, matched as? LinearLayout) },
        "ColorSwatch" to { data, matched -> createColorSwatch(data, matched) },
        "PdfPagePicker" to { data, matched -> createPdfPagePicker(data, matched as? HorizontalScrollView) },
        "SignaturePad" to { data, matched -> createSignaturePad(data, matched as? SignaturePadView) },
        "PdfSignPlacement" to { data, matched -> createPdfSignPlacement(data, matched as? SignPlacementView) },
        "PdfSignPreview" to { data, matched -> createPdfSignPreview(data, matched as? PdfSignPreview) },
        "PdfPreviewGrid" to { data, matched -> createPdfPreviewGrid(data, matched as? ScrollView) },
        "PdfSinglePage" to { data, matched -> createPdfSinglePage(data, matched as? ImageView) },
        "CodeView" to { data, matched -> createCodeView(data, matched as? WebView) },
        "HtmlView" to { data, matched -> createHtmlView(data, matched as? WebView) },
        "Compass" to { data, matched -> createCompass(data, matched) },
        "Barometer" to { data, matched -> createBarometer(data, matched as? SensorShaderView) },
        "Magnetometer" to { data, matched -> createMagnetometer(data, matched as? SensorShaderView) },
    )
    private val host = FrameLayout(context).apply {
        layoutParams = FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        )
    }
    private val mainHandler = Handler(Looper.getMainLooper())
    private var currentRoot: View? = null
    private var pooledCodeView: WebView? = null
    private var lastFindQuery: String = ""
    private var findStatusView: TextView? = null
    private val bindings = mutableMapOf<String, String>()
    private val pendingBindingUpdates = mutableMapOf<String, Runnable>()
    private val allowedTypes = setOf(
        "Column",
        "Section",
        "Card",
        "Text",
        "Button",
        "ShaderToy",
        "TextInput",
        "Checkbox",
        "Progress",
        "Grid",
        "ImageBase64",
        "ColorSwatch",
        "PdfPagePicker",
        "SignaturePad",
        "PdfSignPlacement",
        "PdfSignPreview",
        "PdfPreviewGrid",
        "PdfSinglePage",
        "CodeView",
        "HtmlView",
        "Compass",
        "Barometer",
        "Magnetometer",
        "VirtualList"
    )

    fun render(jsonString: String): View {
        bindings.clear()
        findStatusView = null
        val rootJson = try {
            JSONObject(jsonString)
        } catch (e: Exception) {
            return setHostContent(renderFallback("Render error", "Invalid JSON"))
        }

        val validationError = validate(rootJson)
        if (validationError != null) {
            return setHostContent(renderFallback("Render error", validationError))
        }

        val root = createRoot(rootJson)
        return setHostContent(root)
    }

    fun renderFallback(title: String, message: String): View {
        val layout = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        val padding = dpToPx(context, 16f)
        layout.setPadding(padding, padding, padding, padding)

        layout.addView(TextView(context).apply {
            text = title
            textSize = 18f
            setTextColor(Color.RED)
        })

        layout.addView(TextView(context).apply {
            text = message
            textSize = 14f
        })

        layout.addView(Button(context).apply {
            text = "Back"
            setOnClickListener { onAction("reset", false, false, emptyMap()) }
        })

        return ScrollView(context).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            addView(layout)
        }
    }

    private fun setHostContent(content: View): View {
        detachFromParent(content, host)
        if (host.childCount == 0 || host.getChildAt(0) !== content) {
            host.removeAllViews()
            host.addView(content)
        }
        currentRoot = content
        return host
    }

    private fun createRoot(data: JSONObject): View {
        val scrollable = data.optBoolean("scrollable", true)
        val existingRoot = currentRoot
        val existingContent = when {
            scrollable -> (existingRoot as? ScrollView)?.getChildAt(0)
            existingRoot is ScrollView -> null
            else -> existingRoot
        }
        val content = createView(data, existingContent)
        return if (scrollable) {
            val scroll = (existingRoot as? ScrollView) ?: ScrollView(context).apply {
                layoutParams = FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.MATCH_PARENT,
                    FrameLayout.LayoutParams.MATCH_PARENT
                )
                isFillViewport = true
            }
            content.layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            scroll.layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            if (scroll.childCount == 0 || scroll.getChildAt(0) !== content) {
                scroll.removeAllViews()
                scroll.addView(content)
            }
            scroll
        } else {
            content.layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            content
        }
    }

    private fun createView(data: JSONObject, existing: View? = null): View {
        val type = data.optString("type", "")
        val nodeId = resolveNodeId(data)
        val matched = if (existing != null && matchesMeta(existing, type, nodeId)) existing else null
        val creator = creators[type] ?: return createErrorView(if (type.isBlank()) "Missing type" else "Unknown: $type")
        return creator.invoke(data, matched)
    }

    private fun validate(node: JSONObject): String? {
        val type = node.optString("type", "")
        if (type.isBlank()) return "Missing type"
        if (!allowedTypes.contains(type)) return "Unknown widget: $type"
        if ((type == "Column" || type == "Grid" || type == "Section" || type == "Card") && !node.has("children")) {
            return "$type missing children"
        }
        if (type == "ImageBase64" && !node.has("base64")) {
            return "ImageBase64 missing base64"
        }
        if (type == "ColorSwatch" && !node.has("color")) {
            return "ColorSwatch missing color"
        }
        if (type == "Button") {
            if (!node.has("text")) return "Button missing text"
            val hasAction = node.has("action")
            val hasCopy = node.has("copy_text")
            if (!hasAction && !hasCopy) return "Button missing action or copy_text"
        }
        if (type == "Text" && !node.has("text")) {
            return "Text missing text"
        }
        if (type == "TextInput" && !node.has("bind_key")) {
            return "TextInput missing bind_key"
        }
        if (type == "Checkbox" && !node.has("bind_key")) {
            return "Checkbox missing bind_key"
        }
        if (type == "PdfPagePicker") {
            if (!node.has("page_count")) return "PdfPagePicker missing page_count"
            if (!node.has("source_uri")) return "PdfPagePicker missing source_uri"
            if (!node.has("bind_key")) return "PdfPagePicker missing bind_key"
        }
        if (type == "SignaturePad") {
            if (!node.has("bind_key")) return "SignaturePad missing bind_key"
        }
        if (type == "PdfSignPlacement") {
            if (!node.has("source_uri")) return "PdfSignPlacement missing source_uri"
            if (!node.has("page_count")) return "PdfSignPlacement missing page_count"
        }
        if (type == "PdfSignPreview") {
            if (!node.has("page_count")) return "PdfSignPreview missing page_count"
            if (!node.has("bind_key_page")) return "PdfSignPreview missing bind_key_page"
            if (!node.has("bind_key_x_pct")) return "PdfSignPreview missing bind_key_x_pct"
            if (!node.has("bind_key_y_pct")) return "PdfSignPreview missing bind_key_y_pct"
            if (!node.has("source_uri")) return "PdfSignPreview missing source_uri"
        }
        if (type == "PdfPreviewGrid") {
            if (!node.has("source_uri")) return "PdfPreviewGrid missing source_uri"
            if (!node.has("page_count")) return "PdfPreviewGrid missing page_count"
            if (!node.has("action")) return "PdfPreviewGrid missing action"
        }
        if (type == "PdfSinglePage") {
            if (!node.has("source_uri")) return "PdfSinglePage missing source_uri"
            if (!node.has("page")) return "PdfSinglePage missing page"
        }
        if (type == "CodeView" && !node.has("text")) {
            return "CodeView missing text"
        }
        if (type == "Compass" && !node.has("angle_radians")) {
            return "Compass missing angle_radians"
        }
        if (type == "Barometer" && !node.has("hpa")) {
            return "Barometer missing hpa"
        }
        if (type == "Magnetometer" && !node.has("magnitude_ut")) {
            return "Magnetometer missing magnitude_ut"
        }
        if (type == "Grid" || type == "Column" || type == "Section" || type == "Card" || type == "VirtualList") {
            val children = node.optJSONArray("children") ?: return "$type missing children"
            for (i in 0 until children.length()) {
                val childErr = validate(children.getJSONObject(i))
                if (childErr != null) return childErr
            }
        }
        return null
    }

    // WARNING: For createColumn, make sure to call createView recursively
    // I'm putting the abbreviated code back for clarity:
    private fun createColumn(data: JSONObject, existing: LinearLayout?): View {
        val layout = existing ?: LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        layout.orientation = LinearLayout.VERTICAL
        val focusedInput = layout.findFocus() as? EditText
        val selection = focusedInput?.selectionStart ?: -1
        val padding = data.optInt("padding", 0)
        layout.setPadding(padding, padding, padding, padding)
        val contentDescription = data.optString("content_description", "")
        layout.contentDescription = contentDescription.takeIf { it.isNotEmpty() }
        val children = data.optJSONArray("children")
        val newChildren = mutableListOf<View>()
        if (children != null) {
            for (i in 0 until children.length()) {
                val childJson = children.getJSONObject(i)
                val reuse = existing?.let { findReusableChild(it, childJson) }
                val childView = createView(childJson, reuse)
                detachFromParent(childView, layout)
                newChildren.add(childView)
            }
        } else {
            newChildren.add(createErrorView("Missing children"))
        }
        layout.removeAllViews()
        newChildren.forEach { layout.addView(it) }
        if (focusedInput != null && focusedInput.parent != null) {
            focusedInput.requestFocus()
            if (selection >= 0) {
                val len = focusedInput.text?.length ?: 0
                val pos = selection.coerceAtMost(len)
                focusedInput.setSelection(pos)
            }
        }
        setMeta(layout, "Column", resolveNodeId(data))
        return layout
    }

    private fun createVirtualList(data: JSONObject, existing: LinearLayout?): View {
        // For now, render similarly to Column; VirtualList semantics are handled by Rust-side paging.
        return createColumn(data, existing)
    }

    private fun createSection(data: JSONObject, existing: LinearLayout?): View {
        return createContainerWithHeader(
            data = data,
            existing = existing,
            type = "Section",
            backgroundColor = Color.parseColor("#f6f7fb"),
            strokeColor = Color.parseColor("#e4e7ee"),
            cornerRadiusDp = 12f,
            elevationDp = 0f
        )
    }

    private fun createCard(data: JSONObject, existing: LinearLayout?): View {
        return createContainerWithHeader(
            data = data,
            existing = existing,
            type = "Card",
            backgroundColor = Color.WHITE,
            strokeColor = Color.parseColor("#dfe3eb"),
            cornerRadiusDp = 14f,
            elevationDp = 2f
        )
    }

    private fun createContainerWithHeader(
        data: JSONObject,
        existing: LinearLayout?,
        type: String,
        backgroundColor: Int,
        strokeColor: Int,
        cornerRadiusDp: Float,
        elevationDp: Float
    ): View {
        val layout = existing ?: LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        layout.orientation = LinearLayout.VERTICAL
        val focusedInput = layout.findFocus() as? EditText
        val selection = focusedInput?.selectionStart ?: -1
        val padPx = dpToPx(context, data.optInt("padding", 12).toFloat())
        layout.setPadding(padPx, padPx, padPx, padPx)
        layout.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT).apply {
            topMargin = dpToPx(context, 8f)
            bottomMargin = dpToPx(context, 8f)
        }
        val background = GradientDrawable().apply {
            cornerRadius = dpToPx(context, cornerRadiusDp).toFloat()
            setColor(backgroundColor)
            setStroke(dpToPx(context, 1f), strokeColor)
        }
        layout.background = background
        layout.elevation = dpToPx(context, elevationDp).toFloat()
        val cd = data.optString("content_description", "")
        layout.contentDescription = cd.takeIf { it.isNotEmpty() }

        val children = data.optJSONArray("children")
        val newChildren = mutableListOf<View>()
        if (children != null) {
            for (i in 0 until children.length()) {
                val childJson = children.getJSONObject(i)
                val reuse = existing?.let { findReusableChild(it, childJson) }
                val childView = createView(childJson, reuse)
                detachFromParent(childView, layout)
                newChildren.add(childView)
            }
        } else {
            newChildren.add(createErrorView("$type missing children"))
        }

        layout.removeAllViews()
        buildHeaderView(data)?.let { layout.addView(it) }
        newChildren.forEach { layout.addView(it) }
        if (focusedInput != null && focusedInput.parent != null) {
            focusedInput.requestFocus()
            if (selection >= 0) {
                val len = focusedInput.text?.length ?: 0
                val pos = selection.coerceAtMost(len)
                focusedInput.setSelection(pos)
            }
        }
        setMeta(layout, type, resolveNodeId(data))
        return layout
    }

    private fun buildHeaderView(data: JSONObject): View? {
        val title = data.optString("title", "").takeIf { it.isNotBlank() } ?: return null
        val subtitle = data.optString("subtitle", "")
        val icon = data.optString("icon", "")
        val row = LinearLayout(context).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
            setPadding(0, 0, 0, dpToPx(context, 6f))
        }
        if (icon.isNotEmpty()) {
            row.addView(TextView(context).apply {
                text = icon
                textSize = 18f
                setPadding(0, 0, dpToPx(context, 8f), 0)
            })
        }
        val textCol = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        textCol.addView(TextView(context).apply {
            text = title
            textSize = 16f
        })
        if (subtitle.isNotBlank()) {
            textCol.addView(TextView(context).apply {
                text = subtitle
                textSize = 12f
                setTextColor(Color.parseColor("#5f6372"))
            })
        }
        row.addView(textCol)
        return row
    }

    private fun createImageBase64(data: JSONObject, existing: LinearLayout?): View {
        val b64 = data.optString("base64", "")
        if (b64.isBlank()) return createErrorView("Missing base64")
        val container = existing ?: LinearLayout(context)
        container.orientation = LinearLayout.VERTICAL
        val padding = dpToPx(context, 16f)
        container.setPadding(padding, padding, padding, padding)
        container.setBackgroundColor(Color.WHITE)
        container.elevation = dpToPx(context, 2f).toFloat()
        container.layoutParams = LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.MATCH_PARENT,
            LinearLayout.LayoutParams.WRAP_CONTENT
        ).apply {
            topMargin = dpToPx(context, 12f)
            bottomMargin = dpToPx(context, 12f)
        }
        val iv = (container.getChildAt(0) as? android.widget.ImageView) ?: android.widget.ImageView(context).apply {
            scaleType = android.widget.ImageView.ScaleType.FIT_CENTER
            adjustViewBounds = true // let it scale to available width
        }
        val lastB64 = container.getTag(dataTag) as? String
        if (lastB64 != b64) {
            val bytes = try {
                android.util.Base64.decode(b64, android.util.Base64.DEFAULT)
            } catch (_: Exception) {
                null
            } ?: return createErrorView("Invalid base64")
            val bmp = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
            iv.setImageBitmap(bmp)
            container.setTag(dataTag, b64)
        }
        val cd = data.optString("content_description", "")
        iv.contentDescription = cd.takeIf { it.isNotEmpty() }
        if (iv.parent == null) {
            container.addView(iv)
        }
        setMeta(container, "ImageBase64", resolveNodeId(data))
        return container
    }

    private fun createColorSwatch(data: JSONObject, existing: View?): View {
        val colorLong = data.optLong("color", 0xFF000000)
        val view = existing ?: View(context)
        val size = dpToPx(context, 128f)
        val lp = LinearLayout.LayoutParams(size, size)
        lp.topMargin = dpToPx(context, 8f)
        lp.bottomMargin = dpToPx(context, 8f)
        view.layoutParams = lp
        view.setBackgroundColor(colorLong.toInt())
        val cd = data.optString("content_description", "")
        view.contentDescription = cd.takeIf { it.isNotEmpty() }
        setMeta(view, "ColorSwatch", resolveNodeId(data))
        return view
    }

    private fun createCompass(data: JSONObject, existing: View?): View {
        val angle = data.optDouble("angle_radians", 0.0).toFloat()
        val cd = data.optString("content_description", "").takeIf { it.isNotBlank() }
        val size = dpToPx(context, 280f)

        val glView = existing as? CompassGLView ?: CompassGLView(context)
        glView.layoutParams = LayoutParams(size, size)
        glView.contentDescription = cd
        glView.setAngle(angle)
        val view: View = glView
        setMeta(view, "Compass", resolveNodeId(data))
        return view
    }

    private fun createBarometer(data: JSONObject, existing: SensorShaderView?): View {
        val hpa = data.optDouble("hpa", 0.0).toFloat()
        val view = existing ?: SensorShaderView(context, BAROMETER_FRAGMENT, "u_value")
        val size = dpToPx(context, 220f)
        view.layoutParams = LayoutParams(size, size)
        view.setValue(hpa)
        view.contentDescription = data.optString("content_description", "").takeIf { it.isNotBlank() }
        setMeta(view, "Barometer", resolveNodeId(data))
        return view
    }

    private fun createMagnetometer(data: JSONObject, existing: SensorShaderView?): View {
        val mag = data.optDouble("magnitude_ut", 0.0).toFloat()
        val view = existing ?: SensorShaderView(context, MAGNETOMETER_FRAGMENT, "u_value")
        val size = dpToPx(context, 220f)
        view.layoutParams = LayoutParams(size, size)
        view.setValue(mag)
        view.contentDescription = data.optString("content_description", "").takeIf { it.isNotBlank() }
        setMeta(view, "Magnetometer", resolveNodeId(data))
        return view
    }

    private fun createText(data: JSONObject, existing: TextView?): View {
        val view = existing ?: TextView(context)
        view.text = data.optString("text")
        view.textSize = data.optDouble("size", 14.0).toFloat()
        val contentDescription = data.optString("content_description", "")
        view.contentDescription = contentDescription.takeIf { it.isNotEmpty() }
        val nodeId = resolveNodeId(data)
        if (nodeId == "find_status") {
            findStatusView = view
        }
        setMeta(view, "Text", nodeId)
        return view
    }

    private fun createCodeView(data: JSONObject, existing: WebView?): View {
        val text = data.optString("text", "")
        val language = data.optString("language", "none").ifBlank { "none" }
        val wrap = data.optBoolean("wrap", true)
        val theme = data.optString("theme", "light")
        val contentDescription = data.optString("content_description", "")

        val webView = existing ?: pooledCodeView ?: WebView(context).also { pooledCodeView = it }
        configureCodeWebView(webView, wrap, contentDescription)

        val escaped = escapeHtml(text)
        val background = if (theme == "dark") "#0f111a" else "#fafafa"
        val foreground = if (theme == "dark") "#e6e6e6" else "#1a1a1a"
        val wrapClass = if (wrap) "wrap" else "nowrap"
        val lineNumbers = data.optBoolean("line_numbers", false)
        val lineClass = if (lineNumbers) "line-numbers" else ""
        val html = """
            <!DOCTYPE html>
            <html>
            <head>
              <meta charset="utf-8" />
              <meta name="viewport" content="width=device-width,initial-scale=1" />
              <style>
                body { margin: 0; padding: 12px; background: $background; color: $foreground; }
                pre { margin: 0; font-family: 'JetBrains Mono', 'SFMono-Regular', Menlo, Consolas, monospace; font-size: 14px; line-height: 1.4; }
                pre.wrap code { white-space: pre-wrap; word-break: break-word; }
                pre.nowrap { overflow-x: auto; }
                code { display: block; }
              </style>
            </head>
            <body>
              <pre class="$wrapClass $lineClass"><code class="language-$language">$escaped</code></pre>
              <script src="prism-bundle.min.js"></script>
              <script>if(window.Prism){Prism.manual=false;Prism.highlightAll();}</script>
            </body>
            </html>
        """.trimIndent()

        val lastHtml = webView.getTag(dataTag) as? String
        if (lastHtml != html) {
            webView.setTag(dataTag, html)
            webView.loadDataWithBaseURL(
                "file:///android_asset/prism/",
                html,
                "text/html",
                "utf-8",
                null
            )
        }
        webView.setFindListener { active, total, done ->
            if (done) {
                if (total <= 0) {
                    updateFindStatus("No matches")
                } else {
                    updateFindStatus("${active + 1} / $total")
                }
            }
        }

        val lp = LayoutParams(LayoutParams.MATCH_PARENT, 0, 1f)
        val margin = dpToPx(context, 8f)
        lp.topMargin = margin
        lp.bottomMargin = margin
        webView.layoutParams = lp

        setMeta(webView, "CodeView", resolveNodeId(data))
        pooledCodeView = webView
        return webView
    }

    private fun createHtmlView(data: JSONObject, existing: WebView?): View {
        val html = data.optString("html", "")
        if (html.isBlank()) return createErrorView("Missing html")
        val heightDp = data.optInt("height_dp", 0)
        val webView = existing ?: WebView(context)
        val settings: WebSettings = webView.settings
        settings.javaScriptEnabled = false
        settings.domStorageEnabled = false
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = true
        settings.displayZoomControls = false

        val lastHtml = webView.getTag(dataTag) as? String
        if (lastHtml != html) {
            webView.setTag(dataTag, html)
            webView.loadDataWithBaseURL(
                null,
                html,
                "text/html",
                "utf-8",
                null
            )
        }

        val lp = LayoutParams(
            LayoutParams.MATCH_PARENT,
            if (heightDp > 0) dpToPx(context, heightDp.toFloat()) else LayoutParams.WRAP_CONTENT
        )
        val margin = dpToPx(context, 8f)
        lp.topMargin = margin
        lp.bottomMargin = margin
        webView.layoutParams = lp
        setMeta(webView, "HtmlView", resolveNodeId(data))
        return webView
    }

    fun performTextFind(query: String, direction: String?) {
        val webView = pooledCodeView ?: return
        val trimmed = query.trim()
        if (trimmed != lastFindQuery) {
            if (trimmed.isEmpty()) {
                webView.clearMatches()
                updateFindStatus("Cleared search")
            } else {
                updateFindStatus("Searching…")
                webView.findAllAsync(trimmed)
            }
            lastFindQuery = trimmed
        }
        when (direction) {
            "next" -> webView.findNext(true)
            "prev" -> webView.findNext(false)
        }
    }

    private fun updateFindStatus(text: String) {
        val target = findStatusView ?: return
        if (Looper.myLooper() == Looper.getMainLooper()) {
            target.text = text
        } else {
            mainHandler.post { target.text = text }
        }
    }

    private fun createButton(data: JSONObject, existing: Button?): View {
        val btn = existing ?: Button(context)
        btn.text = data.optString("text")
        val contentDescription = data.optString("content_description", "")
        btn.contentDescription = contentDescription.takeIf { it.isNotEmpty() }

        // Retrieve the action defined in the Rust JSON (e.g., "hash_file")
        val actionName = data.optString("action")
        val needsFilePicker = data.optBoolean("requires_file_picker", false)
        val allowMultipleFiles = data.optBoolean("allow_multiple_files", false)
        val copyText = data.optString("copy_text", "")
        val payload = data.optJSONObject("payload")

        btn.setOnClickListener {
            flushPendingBindings()
            if (copyText.isNotEmpty()) {
                copyToClipboard(copyText)
            }
            if (actionName.isNotEmpty()) {
                val merged = bindings.toMutableMap()
                if (payload != null) {
                    val keys = payload.keys()
                    while (keys.hasNext()) {
                        val k = keys.next()
                        val v = payload.optString(k, "")
                        merged[k] = v
                    }
                }
                onAction(actionName, needsFilePicker, allowMultipleFiles, merged.toMap())
            }
        }
        setMeta(btn, "Button", resolveNodeId(data))
        return btn
    }

    private fun createErrorView(msg: String): View {
        return TextView(context).apply {
            text = msg
            setTextColor(Color.RED)
        }
    }

    private fun createTextInput(data: JSONObject, existing: EditText?): View {
        val editText = existing ?: EditText(context)
        val bindKey = data.optString("bind_key", "")
        val initial = data.optString("text", "")
        val hasExplicitText = data.has("text")
        val debounceMs = data.optLong("debounce_ms", 0L).coerceAtLeast(0L)
        if (hasExplicitText && editText.text.toString() != initial) {
            editText.setText(initial)
        } else if (!hasExplicitText && existing == null) {
            editText.setText(initial)
        }
        if (bindKey.isNotEmpty()) {
            bindings[bindKey] = editText.text?.toString().orEmpty()
        }
        val hint = data.optString("hint", "")
        editText.hint = hint.takeIf { it.isNotEmpty() }
        val contentDescription = data.optString("content_description", "")
        editText.contentDescription = contentDescription.takeIf { it.isNotEmpty() }

        val singleLine = data.optBoolean("single_line", false)
        editText.isSingleLine = singleLine
        val maxLines = data.optInt("max_lines", 0)
        editText.maxLines = if (maxLines > 0) maxLines else Int.MAX_VALUE
        val mask = data.optBoolean("password_mask", false)
        if (mask) {
            editText.transformationMethod = PasswordTransformationMethod.getInstance()
            editText.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        } else {
            editText.transformationMethod = null
            if (editText.inputType == (InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)) {
                editText.inputType = InputType.TYPE_CLASS_TEXT
            }
        }

        val submitAction = data.optString("action_on_submit", "")
        if (bindKey.isNotEmpty() && editText.getTag(bindKeyTag) != bindKey) {
            editText.addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) {
                    val changeAction = if (debounceMs > 0 && submitAction.isNotEmpty()) {
                        submitAction
                    } else {
                        null
                    }
                    val delay = if (changeAction != null && debounceMs > 0) debounceMs else 120L
                    scheduleBindingUpdate(
                        bindKey,
                        s?.toString().orEmpty(),
                        changeAction,
                        false,
                        false,
                        delay
                    )
                }

                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
            })
            editText.setTag(bindKeyTag, bindKey)
        }

        if (submitAction.isNotEmpty()) {
            editText.setOnEditorActionListener { _, actionId, _ ->
                val isDone = actionId == EditorInfo.IME_ACTION_DONE || actionId == EditorInfo.IME_NULL
                if (isDone) {
                    flushPendingBindings()
                    onAction(submitAction, false, false, bindings.toMap())
                }
                isDone
            }
        } else {
            editText.setOnEditorActionListener(null)
        }
        if (bindKey.isNotEmpty()) {
            editText.onFocusChangeListener = View.OnFocusChangeListener { _, hasFocus ->
                if (!hasFocus) {
                    scheduleBindingUpdate(bindKey, editText.text?.toString().orEmpty())
                    flushPendingBindings()
                }
            }
        } else {
            editText.onFocusChangeListener = null
        }
        setMeta(editText, "TextInput", resolveNodeId(data))
        return editText
    }

    private fun createShaderToy(data: JSONObject, existing: ShaderToyView?): View {
        val fragment = data.optString("fragment", DEFAULT_FRAGMENT)
        val existingFragment = existing?.getTag(dataTag) as? String
        val view = if (existing != null && existingFragment == fragment) {
            existing
        } else {
            ShaderToyView(context, fragment).apply { setTag(dataTag, fragment) }
        }
        val lp = LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.MATCH_PARENT,
            dpToPx(context, 240f)
        )
        val margin = dpToPx(context, 12f)
        lp.topMargin = margin
        lp.bottomMargin = margin
        view.layoutParams = lp
        val contentDescription = data.optString("content_description", "")
        view.contentDescription = contentDescription.takeIf { it.isNotEmpty() }
        setMeta(view, "ShaderToy", resolveNodeId(data))
        return view
    }

    private fun createProgress(data: JSONObject, existing: LinearLayout?): View {
        val container = existing ?: LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
        }
        container.orientation = LinearLayout.VERTICAL
        container.layoutParams = LayoutParams(
            LayoutParams.MATCH_PARENT,
            LayoutParams.WRAP_CONTENT
        )
        container.removeAllViews()
        val bar = ProgressBar(context).apply {
            isIndeterminate = true
        }
        val text = data.optString("text", "")
        if (text.isNotEmpty()) {
            container.addView(TextView(context).apply {
                this.text = text
                textSize = 14f
                val margin = dpToPx(context, 8f)
                val lp = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT)
                lp.bottomMargin = margin
                layoutParams = lp
            })
        }
        container.addView(bar)
        val contentDescription = data.optString("content_description", "")
        container.contentDescription = contentDescription.takeIf { it.isNotEmpty() }
        setMeta(container, "Progress", resolveNodeId(data))
        return container
    }

    private fun createCheckbox(data: JSONObject, existing: CheckBox?): View {
        val checkBox = existing ?: CheckBox(context)
        val text = data.optString("text", data.optString("label", ""))
        checkBox.text = text

        val bindKey = data.optString("bind_key", "")
        val checked = data.optBoolean("checked", false)
        checkBox.isChecked = checked
        if (bindKey.isNotEmpty()) {
            bindings[bindKey] = checked.toString()
        }

        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            checkBox.contentDescription = contentDescription
        }

        val actionName = data.optString("action", "")
        val needsFilePicker = data.optBoolean("requires_file_picker", false)
        if (bindKey.isNotEmpty()) {
            checkBox.setOnCheckedChangeListener { _, isChecked ->
                bindings[bindKey] = isChecked.toString()
                flushPendingBindings()
                if (actionName.isNotEmpty()) {
                    onAction(actionName, needsFilePicker, false, bindings.toMap())
                }
            }
        }
        setMeta(checkBox, "Checkbox", resolveNodeId(data))
        return checkBox
    }

    private fun createGrid(data: JSONObject, existing: LinearLayout?): View {
        val columns = computeColumns(data)
        val children = data.optJSONArray("children") ?: return createErrorView("Grid missing children")
        val wrapper = existing ?: LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
        }
        wrapper.orientation = LinearLayout.VERTICAL
        wrapper.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
        val padding = data.optInt("padding", 0)
        wrapper.setPadding(padding, padding, padding, padding)
        val contentDescription = data.optString("content_description", "")
        wrapper.contentDescription = contentDescription.takeIf { it.isNotEmpty() }

        val rows = mutableListOf<LinearLayout>()
        var row: LinearLayout? = null
        for (i in 0 until children.length()) {
            val childJson = children.getJSONObject(i)
            if (i % columns == 0) {
                row = LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
                }
                rows.add(row)
            }
            val reuse = existing?.let { findReusableChild(it, childJson) }
            val childView = createView(childJson, reuse)
            detachFromParent(childView, row ?: wrapper)
            val lp = LinearLayout.LayoutParams(0, LayoutParams.WRAP_CONTENT, 1f)
            childView.layoutParams = lp
            row?.addView(childView)
        }
        wrapper.removeAllViews()
        rows.forEach { wrapper.addView(it) }
        setMeta(wrapper, "Grid", resolveNodeId(data))
        return wrapper
    }

    private fun createPdfPagePicker(data: JSONObject, existing: HorizontalScrollView?): View {
        val pageCount = data.optInt("page_count", 0)
        val bindKey = data.optString("bind_key", "")
        val sourceUri = data.optString("source_uri", "")
        if (pageCount <= 0 || sourceUri.isBlank()) return createErrorView("PDF picker missing data")
        val uri = runCatching { Uri.parse(sourceUri) }.getOrNull() ?: return createErrorView("Invalid PDF URI")

        val selected = mutableSetOf<Int>()
        val selectedArr = data.optJSONArray("selected_pages")
        if (selectedArr != null) {
            for (i in 0 until selectedArr.length()) {
                val valNum = selectedArr.optInt(i, -1)
                if (valNum > 0) selected.add(valNum)
            }
        }

        fun pushSelection() {
            if (bindKey.isNotEmpty()) {
                bindings[bindKey] = selected.sorted().joinToString(",")
            }
        }

        val scroller = existing ?: HorizontalScrollView(context)
        scroller.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
        val strip = (scroller.getChildAt(0) as? LinearLayout) ?: LinearLayout(context).apply {
            orientation = LinearLayout.HORIZONTAL
            layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT)
        }
        val pad = dpToPx(context, 8f)
        strip.setPadding(pad, pad, pad, pad)
        val cd = data.optString("content_description", "")
        strip.contentDescription = cd.takeIf { it.isNotEmpty() }

        val cached = scroller.getTag(dataTag) as? PdfPickerCache
        val canReuseThumbs = cached?.uri == sourceUri && cached.pageCount == pageCount && strip.childCount == pageCount
        val thumbnails = if (canReuseThumbs) null else renderPdfThumbnails(uri, pageCount)
        if (!canReuseThumbs) {
            strip.removeAllViews()
        }

        for (i in 0 until pageCount) {
            val pageNumber = i + 1
            val existingCell = if (canReuseThumbs) strip.getChildAt(i) as? LinearLayout else null
            val cell = existingCell ?: LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT).apply {
                    marginEnd = dpToPx(context, 10f)
                }
            }
            if (!canReuseThumbs) {
                cell.removeAllViews()
                val thumb = thumbnails?.getOrNull(i)
                if (thumb != null) {
                    val iv = ImageView(context).apply {
                        setImageBitmap(thumb)
                        adjustViewBounds = true
                        scaleType = ImageView.ScaleType.CENTER_CROP
                        layoutParams = LayoutParams(dpToPx(context, 140f), LayoutParams.WRAP_CONTENT).apply {
                            bottomMargin = dpToPx(context, 6f)
                        }
                    }
                    cell.addView(iv)
                } else {
                    cell.addView(createErrorView("Preview $pageNumber"))
                }
            }
            val check = (cell.findViewWithTag<View>("pdf_check_$pageNumber") as? CheckBox) ?: CheckBox(context).apply {
                tag = "pdf_check_$pageNumber"
                cell.addView(this)
            }
            check.text = "Page $pageNumber"
            check.isChecked = selected.contains(pageNumber)
            check.setOnCheckedChangeListener { _, isChecked ->
                if (isChecked) selected.add(pageNumber) else selected.remove(pageNumber)
                pushSelection()
            }
            if (cell.parent == null) {
                strip.addView(cell)
            }
        }

        pushSelection()
        if (strip.parent != scroller) {
            scroller.removeAllViews()
            scroller.addView(strip)
        }
        scroller.setTag(dataTag, PdfPickerCache(sourceUri, pageCount))
        setMeta(scroller, "PdfPagePicker", resolveNodeId(data))
        return scroller
    }

    private fun renderPdfThumbnails(uri: Uri, pageCount: Int): List<Bitmap?> {
        val thumbs = mutableListOf<Bitmap?>()
        val pfd: ParcelFileDescriptor = try {
            context.contentResolver.openFileDescriptor(uri, "r") ?: return List(pageCount) { null }
        } catch (_: Exception) {
            return List(pageCount) { null }
        }
        pfd.use { descriptor ->
            try {
                PdfRenderer(descriptor).use { renderer ->
                    val count = minOf(pageCount, renderer.pageCount)
                    for (i in 0 until count) {
                        thumbs.add(renderSinglePage(renderer, i))
                    }
                    if (count < pageCount) {
                        repeat(pageCount - count) { thumbs.add(null) }
                    }
                }
            } catch (_: Exception) {
                return List(pageCount) { null }
            }
        }
        return thumbs
    }

    private fun renderSinglePage(renderer: PdfRenderer, index: Int): Bitmap? {
        if (index >= renderer.pageCount) return null
        return try {
            renderer.openPage(index).use { page ->
                val targetWidth = dpToPx(context, 140f)
                val aspect = page.height / page.width.toFloat()
                val targetHeight = (targetWidth * aspect).toInt().coerceAtLeast(24)
                val bmp = Bitmap.createBitmap(targetWidth, targetHeight, Bitmap.Config.ARGB_8888)
                page.render(bmp, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                bmp
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun createPdfPreviewGrid(data: JSONObject, existing: ScrollView?): View {
        val sourceUri = data.optString("source_uri", "")
        val pageCount = data.optInt("page_count", 0)
        val actionName = data.optString("action", "")
        val uri = try {
            Uri.parse(sourceUri)
        } catch (_: Exception) {
            return createErrorView("Invalid source_uri")
        }
        val container = existing ?: ScrollView(context)
        val grid = (container.getChildAt(0) as? GridLayout) ?: GridLayout(context).apply {
            columnCount = 2
            layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
        }
        grid.removeAllViews()

        val thumbs = if (pageCount > 0) renderPdfThumbnails(uri, pageCount) else emptyList()
        for (i in 0 until pageCount) {
            val pageIndex = i + 1
            val cell = LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                val pad = dpToPx(context, 8f)
                setPadding(pad, pad, pad, pad)
                layoutParams = GridLayout.LayoutParams().apply {
                    width = 0
                    height = ViewGroup.LayoutParams.WRAP_CONTENT
                    columnSpec = GridLayout.spec(GridLayout.UNDEFINED, 1f)
                }
            }
            val thumb = thumbs.getOrNull(i)
            if (thumb != null) {
                val iv = ImageView(context).apply {
                    setImageBitmap(thumb)
                    adjustViewBounds = true
                    scaleType = ImageView.ScaleType.CENTER_CROP
                    layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
                }
                cell.addView(iv)
            } else {
                cell.addView(createErrorView("Page $pageIndex"))
            }
            val btn = Button(context).apply {
                text = "Page $pageIndex"
                setOnClickListener {
                    flushPendingBindings()
                    if (actionName.isNotEmpty()) {
                        onAction(actionName, false, false, mapOf("page" to pageIndex.toString()))
                    }
                }
            }
            cell.addView(btn)
            grid.addView(cell)
        }
        if (grid.parent != container) {
            container.removeAllViews()
            container.addView(grid)
        }
        setMeta(container, "PdfPreviewGrid", resolveNodeId(data))
        return container
    }

    private fun createPdfSinglePage(data: JSONObject, existing: ImageView?): View {
        val sourceUri = data.optString("source_uri", "")
        val page = data.optInt("page", 1).coerceAtLeast(1) - 1
        val uri = try {
            Uri.parse(sourceUri)
        } catch (_: Exception) {
            return createErrorView("Invalid source_uri")
        }
        val image = existing ?: ImageView(context)
        image.adjustViewBounds = true
        val bmp = renderPdfPage(uri, page)
        if (bmp == null) return createErrorView("Preview unavailable")
        image.setImageBitmap(bmp)
        image.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
        image.scaleType = ImageView.ScaleType.FIT_CENTER
        setMeta(image, "PdfSinglePage", resolveNodeId(data))
        return image
    }

    private fun renderPdfPage(uri: Uri, index: Int): Bitmap? {
        val pfd: ParcelFileDescriptor = try {
            context.contentResolver.openFileDescriptor(uri, "r") ?: return null
        } catch (_: Exception) {
            return null
        }
        pfd.use { descriptor ->
            return try {
                PdfRenderer(descriptor).use { renderer ->
                    if (index < 0 || index >= renderer.pageCount) return null
                    renderer.openPage(index).use { page ->
                        val targetWidth = dpToPx(context, 320f).coerceAtLeast(160)
                        val aspect = page.height / page.width.toFloat()
                        val targetHeight = (targetWidth * aspect).toInt().coerceAtLeast(120)
                        val bmp = Bitmap.createBitmap(
                            targetWidth,
                            targetHeight,
                            Bitmap.Config.ARGB_8888
                        )
                        page.render(bmp, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                        bmp
                    }
                }
            } catch (_: Exception) {
                null
            }
        }
    }

    private fun createSignaturePad(data: JSONObject, existing: SignaturePadView?): View {
        val bindKey = data.optString("bind_key", "")
        val heightDp = data.optInt("height_dp", 180)
        val cd = data.optString("content_description", "")
        var padRef: SignaturePadView? = existing
        val pad = (existing ?: SignaturePadView(context) { b64, widthPx, heightPx, dpi ->
            padRef?.setTag(dataTag, SignatureState(b64, widthPx, heightPx, dpi))
            if (bindKey.isNotEmpty()) {
                bindings[bindKey] = b64
            }
            bindings["signature_width_px"] = widthPx.toString()
            bindings["signature_height_px"] = heightPx.toString()
            bindings["signature_dpi"] = dpi.toString()
            val widthKey = "pdf_signature_width"
            val heightKey = "pdf_signature_height"
            val needsWidth = bindings[widthKey].isNullOrBlank()
            val needsHeight = bindings[heightKey].isNullOrBlank()
            val pxToPt = if (dpi > 0f) 72f / dpi else 0.0f
            if (needsWidth && pxToPt > 0f) {
                bindings[widthKey] = (widthPx * pxToPt).toString()
            }
            if (needsHeight && pxToPt > 0f) {
                bindings[heightKey] = (heightPx * pxToPt).toString()
            }
        }).also { padRef = it }
        val lp = LayoutParams(LayoutParams.MATCH_PARENT, dpToPx(context, heightDp.toFloat()))
        lp.topMargin = dpToPx(context, 8f)
        lp.bottomMargin = dpToPx(context, 8f)
        pad.layoutParams = lp
        pad.contentDescription = cd.takeIf { it.isNotEmpty() }
        val cached = pad.getTag(dataTag) as? SignatureState
        if (cached != null) {
            if (bindKey.isNotEmpty()) {
                bindings[bindKey] = cached.base64
            }
            bindings["signature_width_px"] = cached.widthPx.toString()
            bindings["signature_height_px"] = cached.heightPx.toString()
            bindings["signature_dpi"] = cached.dpi.toString()
        }
        setMeta(pad, "SignaturePad", resolveNodeId(data))
        return pad
    }

    private fun createPdfSignPlacement(data: JSONObject, existing: SignPlacementView?): View {
        val pageCount = data.optInt("page_count", 0)
        val bindPage = data.optString("bind_key_page", "pdf_signature_page")
        val bindX = data.optString("bind_key_x_pct", "pdf_signature_x_pct")
        val bindY = data.optString("bind_key_y_pct", "pdf_signature_y_pct")
        val sourceUri = data.optString("source_uri", "")
        val selectedPage = data.optInt("selected_page", 1).coerceAtLeast(1)
        val selectedX = if (data.has("selected_x_pct")) data.optDouble("selected_x_pct", Double.NaN).toFloat() else Float.NaN
        val selectedY = if (data.has("selected_y_pct")) data.optDouble("selected_y_pct", Double.NaN).toFloat() else Float.NaN
        if (pageCount <= 0 || sourceUri.isBlank()) return createErrorView("SignPlacement missing data")
        val uri = runCatching { Uri.parse(sourceUri) }.getOrNull() ?: return createErrorView("Invalid PDF URI")
        val view = existing ?: SignPlacementView(context)
        view.bind(uri, pageCount, selectedPage, selectedX, selectedY) { page, nx, ny ->
            bindings[bindPage] = page.toString()
            bindings[bindX] = nx.toString()
            bindings[bindY] = ny.toString()
        }
        val cd = data.optString("content_description", "")
        view.contentDescription = cd.takeIf { it.isNotEmpty() }
        setMeta(view, "PdfSignPlacement", resolveNodeId(data))
        return view
    }

    private fun createPdfSignPreview(data: JSONObject, existing: PdfSignPreview?): View {
        val pageCount = data.optInt("page_count", 0)
        val bindPage = data.optString("bind_key_page", "pdf_signature_page")
        val bindX = data.optString("bind_key_x_pct", "pdf_signature_x_pct")
        val bindY = data.optString("bind_key_y_pct", "pdf_signature_y_pct")
        val sourceUri = data.optString("source_uri", "")
        val selectedPage = data.optInt("selected_page", 1).coerceAtLeast(1)
        val selectedX = data.optDouble("selected_x_pct", 0.5).toFloat()
        val selectedY = data.optDouble("selected_y_pct", 0.5).toFloat()
        if (pageCount <= 0 || sourceUri.isBlank()) return createErrorView("SignPreview missing data")
        val uri = runCatching { Uri.parse(sourceUri) }.getOrNull() ?: return createErrorView("Invalid PDF URI")
        val view = existing ?: PdfSignPreview(context)
        view.bind(uri, pageCount, selectedPage, selectedX, selectedY) { page, nx, ny ->
            bindings[bindPage] = page.toString()
            bindings[bindX] = nx.toString()
            bindings[bindY] = ny.toString()
        }
        setMeta(view, "PdfSignPreview", resolveNodeId(data))
        return view
    }

    private class SignaturePadView(
        context: Context,
        private val onUpdate: (String, Int, Int, Float) -> Unit
    ) : View(context) {
        private val path = Path()
        private val paint = Paint().apply {
            color = Color.BLACK
            style = Paint.Style.STROKE
            strokeWidth = dpToPxInternal(2f).toFloat()
            isAntiAlias = true
            strokeJoin = Paint.Join.ROUND
            strokeCap = Paint.Cap.ROUND
        }

        override fun onDraw(canvas: Canvas) {
            super.onDraw(canvas)
            canvas.drawColor(Color.WHITE)
            canvas.drawPath(path, paint)
        }

        override fun onTouchEvent(event: MotionEvent): Boolean {
            val x = event.x
            val y = event.y
            when (event.action) {
                MotionEvent.ACTION_DOWN -> {
                    parent?.requestDisallowInterceptTouchEvent(true)
                    path.moveTo(x, y)
                }
                MotionEvent.ACTION_MOVE -> {
                    parent?.requestDisallowInterceptTouchEvent(true)
                    path.lineTo(x, y)
                }
                MotionEvent.ACTION_UP -> {
                    path.lineTo(x, y)
                    exportAndSend()
                    parent?.requestDisallowInterceptTouchEvent(false)
                }
                MotionEvent.ACTION_CANCEL -> {
                    parent?.requestDisallowInterceptTouchEvent(false)
                }
            }
            invalidate()
            return true
        }

        private fun exportAndSend() {
            if (width <= 0 || height <= 0) return
            val bmp = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
            val canvas = Canvas(bmp)
            canvas.drawColor(Color.WHITE)
            canvas.drawPath(path, paint)
            val stream = ByteArrayOutputStream()
            bmp.compress(Bitmap.CompressFormat.PNG, 100, stream)
            val b64 = Base64.encodeToString(stream.toByteArray(), Base64.NO_WRAP)
            val dpi = resources.displayMetrics.xdpi.takeIf { it > 0f }
                ?: resources.displayMetrics.densityDpi.toFloat().takeIf { it > 0 }
                ?: 160f
            onUpdate(b64, bmp.width, bmp.height, dpi)
        }

        private fun dpToPxInternal(dp: Float): Int {
            val density = resources.displayMetrics.density
            return (dp * density).toInt()
        }
    }

    private class SignPlacementView(
        context: Context
    ) : LinearLayout(context) {
        private var pageCount: Int = 0
        private var currentPage: Int = 1
        private var sourceUri: Uri? = null
        private var onChange: ((Int, Float, Float) -> Unit)? = null
        private val label = TextView(context).apply {
            textSize = 14f
        }
        private val image = ImageView(context).apply {
            adjustViewBounds = true
            scaleType = ImageView.ScaleType.FIT_CENTER
            layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, dpToPxLocal(320f))
            setBackgroundColor(Color.WHITE)
        }
        private val overlayView = SignOverlay(context, image) { nx, ny ->
            onChange?.invoke(currentPage, nx, ny)
            updateLabel()
        }
        private val frame = FrameLayout(context).apply {
            layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
            addView(image)
            addView(overlayView)
        }

        init {
            orientation = VERTICAL
            val controls = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                val pad = dpToPxLocal(4f)
                setPadding(pad, pad, pad, pad)
            }
            val prev = Button(context).apply {
                text = "<"
                setOnClickListener { changePage(currentPage - 1) }
            }
            val next = Button(context).apply {
                text = ">"
                setOnClickListener { changePage(currentPage + 1) }
            }
            val spacer = Space(context).apply {
                layoutParams = LayoutParams(0, 1, 1f)
            }
            controls.addView(prev)
            controls.addView(spacer)
            controls.addView(label)
            controls.addView(Space(context).apply { layoutParams = LayoutParams(0, 1, 1f) })
            controls.addView(next)
            addView(controls)
            addView(frame)
        }

        fun bind(
            uri: Uri,
            pageCount: Int,
            selectedPage: Int,
            selectedX: Float,
            selectedY: Float,
            onChange: (Int, Float, Float) -> Unit
        ) {
            this.sourceUri = uri
            this.pageCount = pageCount
            this.currentPage = selectedPage.coerceIn(1, pageCount)
            this.onChange = onChange
            val nx = if (selectedX.isFinite()) selectedX else 0.5f
            val ny = if (selectedY.isFinite()) selectedY else 0.5f
            overlayView.setNormalized(nx, ny)
            updateLabel()
            renderPage()
            this.onChange?.invoke(currentPage, nx, ny)
        }

        private fun changePage(target: Int) {
            val clamped = target.coerceIn(1, pageCount)
            if (clamped == currentPage) return
            currentPage = clamped
            overlayView.setNormalized(0.5f, 0.5f)
            updateLabel()
            renderPage()
            onChange?.invoke(currentPage, overlayView.normalizedX(), overlayView.normalizedY())
        }

        private fun updateLabel() {
            label.text = "Tap to place signature • Page $currentPage / $pageCount"
        }

        private fun renderPage() {
            val uri = sourceUri ?: return
            val widthHint = frame.width.takeIf { it > 0 } ?: dpToPxLocal(320f)
            val heightHint = frame.height.takeIf { it > 0 } ?: dpToPxLocal(400f)
            post {
                val pfd = try {
                    context.contentResolver.openFileDescriptor(uri, "r")
                } catch (_: Exception) {
                    null
                } ?: return@post
                pfd.use { descriptor ->
                    try {
                        PdfRenderer(descriptor).use { renderer ->
                            if (renderer.pageCount <= 0) return@use
                            val pageIndex = (currentPage - 1).coerceIn(0, renderer.pageCount - 1)
                            renderer.openPage(pageIndex).use { page ->
                                val targetWidth = widthHint.coerceAtLeast(dpToPxLocal(240f))
                                val targetHeight = (targetWidth.toFloat() / page.width * page.height)
                                    .toInt()
                                    .coerceAtLeast(dpToPxLocal(160f))
                                    .coerceAtMost(heightHint.coerceAtLeast(dpToPxLocal(160f)))
                                val bmp = Bitmap.createBitmap(
                                    targetWidth,
                                    targetHeight,
                                    Bitmap.Config.ARGB_8888
                                )
                                page.render(bmp, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                                image.setImageBitmap(bmp)
                                overlayView.setBitmap(bmp)
                            }
                        }
                    } catch (_: Exception) {
                        // best effort; leave previous image
                    }
                }
            }
        }

        private fun dpToPxLocal(dp: Float): Int {
            val density = resources.displayMetrics.density
            return (dp * density).toInt()
        }
    }

    private class PdfSignPreview(
        context: Context
    ) : FrameLayout(context) {
        private var pageCount: Int = 0
        private var currentPage: Int = 1
        private var sourceUri: Uri? = null
        private var normalizedX: Float = 0.5f
        private var normalizedY: Float = 0.5f
        private var onChange: ((Int, Float, Float) -> Unit)? = null
        private val image = ImageView(context).apply {
            adjustViewBounds = true
            scaleType = ImageView.ScaleType.FIT_CENTER
            layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, dpToPxLocal(180f))
            setBackgroundColor(Color.WHITE)
        }
        private val overlay = SignOverlay(context, image) { nx, ny ->
            normalizedX = nx
            normalizedY = ny
            onChange?.invoke(currentPage, nx, ny)
        }

        init {
            setPadding(dpToPxLocal(4f), dpToPxLocal(4f), dpToPxLocal(4f), dpToPxLocal(4f))
            addView(image)
            addView(overlay)
        }

        fun bind(
            uri: Uri,
            pageCount: Int,
            selectedPage: Int,
            x: Float,
            y: Float,
            onChange: (Int, Float, Float) -> Unit
        ) {
            this.sourceUri = uri
            this.pageCount = pageCount
            this.currentPage = selectedPage.coerceIn(1, pageCount)
            this.normalizedX = x
            this.normalizedY = y
            this.onChange = onChange
            overlay.setNormalized(x, y)
            render()
        }

        private fun render() {
            val uri = sourceUri ?: return
            post {
                val pfd = try {
                    context.contentResolver.openFileDescriptor(uri, "r")
                } catch (_: Exception) {
                    null
                } ?: return@post
                pfd.use { descriptor ->
                    try {
                        PdfRenderer(descriptor).use { renderer ->
                            if (renderer.pageCount <= 0) return@use
                            val pageIndex = (currentPage - 1).coerceIn(0, renderer.pageCount - 1)
                            renderer.openPage(pageIndex).use { page ->
                                val targetWidth = width.takeIf { it > 0 } ?: dpToPxLocal(220f)
                                val targetHeight = (targetWidth.toFloat() / page.width * page.height)
                                    .toInt()
                                    .coerceAtLeast(dpToPxLocal(120f))
                                val bmp = Bitmap.createBitmap(
                                    targetWidth,
                                    targetHeight,
                                    Bitmap.Config.ARGB_8888
                                )
                                page.render(bmp, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                                image.setImageBitmap(bmp)
                                overlay.setBitmap(bmp)
                                overlay.setNormalized(normalizedX, normalizedY)
                            }
                        }
                    } catch (_: Exception) {
                        // best effort; leave previous image
                    }
                }
            }
        }

        private fun dpToPxLocal(dp: Float): Int {
            val density = resources.displayMetrics.density
            return (dp * density).toInt()
        }
    }

    private class SignOverlay(
        context: Context,
        private val image: ImageView,
        private val onTouchUpdate: (Float, Float) -> Unit
    ) : View(context) {
        private var normalizedX: Float? = null
        private var normalizedY: Float? = null
        private val paint = Paint().apply {
            color = Color.RED
            style = Paint.Style.STROKE
            strokeWidth = dpToPxInternal(2f).toFloat()
            isAntiAlias = true
        }

        fun setNormalized(x: Float, y: Float) {
            if (x.isFinite() && y.isFinite()) {
                normalizedX = x
                normalizedY = y
            }
            invalidate()
        }

        fun setBitmap(@Suppress("UNUSED_PARAMETER") bmp: Bitmap) {
            invalidate()
        }

        fun clearMarker() {
            normalizedX = null
            normalizedY = null
            invalidate()
        }

        fun normalizedX(): Float {
            return normalizedX ?: Float.NaN
        }

        fun normalizedY(): Float {
            return normalizedY ?: Float.NaN
        }

        override fun onTouchEvent(event: MotionEvent): Boolean {
            when (event.action) {
                MotionEvent.ACTION_DOWN, MotionEvent.ACTION_MOVE -> {
                    parent?.requestDisallowInterceptTouchEvent(true)
                    mapTouch(event.x, event.y)?.let { (nx, ny) ->
                        normalizedX = nx
                        normalizedY = ny
                        onTouchUpdate(nx, ny)
                        invalidate()
                    }
                }
                MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                    parent?.requestDisallowInterceptTouchEvent(false)
                }
            }
            return true
        }

        override fun onDraw(canvas: Canvas) {
            super.onDraw(canvas)
            val nx = normalizedX
            val ny = normalizedY
            if (nx != null && ny != null) {
                mapNormalizedToView(nx, ny)?.let { (vx, vy) ->
                    canvas.drawCircle(vx, vy, dpToPxInternal(10f).toFloat(), paint)
                    canvas.drawLine(vx - dpToPxInternal(12f), vy, vx + dpToPxInternal(12f), vy, paint)
                    canvas.drawLine(vx, vy - dpToPxInternal(12f), vx, vy + dpToPxInternal(12f), paint)
                }
            }
        }

        private fun mapTouch(x: Float, y: Float): Pair<Float, Float>? {
            val drawable = image.drawable ?: return null
            val matrix = Matrix()
            if (!image.imageMatrix.invert(matrix)) return null
            val pts = floatArrayOf(x, y)
            matrix.mapPoints(pts)
            val w = drawable.intrinsicWidth.toFloat()
            val h = drawable.intrinsicHeight.toFloat()
            if (w <= 0f || h <= 0f) return null
            val nx = (pts[0] / w).coerceIn(0f, 1f)
            val ny = (pts[1] / h).coerceIn(0f, 1f)
            return nx to ny
        }

        private fun mapNormalizedToView(nx: Float, ny: Float): Pair<Float, Float>? {
            val drawable = image.drawable ?: return null
            val w = drawable.intrinsicWidth.toFloat()
            val h = drawable.intrinsicHeight.toFloat()
            if (w <= 0f || h <= 0f) return null
            val pts = floatArrayOf(nx * w, ny * h)
            val matrix = image.imageMatrix
            matrix.mapPoints(pts)
            return pts[0] to pts[1]
        }

        private fun dpToPxInternal(dp: Float): Int {
            val density = resources.displayMetrics.density
            return (dp * density).toInt()
        }
    }

    private fun resolveNodeId(data: JSONObject): String? {
        val explicit = data.optString("id", "").takeIf { it.isNotBlank() }
        if (explicit != null) return explicit
        return when (data.optString("type", "")) {
            "TextInput", "Checkbox", "PdfPagePicker", "SignaturePad", "PdfSignPlacement" ->
                data.optString("bind_key", "").takeIf { it.isNotBlank() }
            "Button" -> data.optString("action", "").takeIf { it.isNotBlank() }
            "Section", "Card" ->
                data.optString("title", "").takeIf { it.isNotBlank() }
                    ?: data.optString("content_description", "").takeIf { it.isNotBlank() }
            "Compass" -> data.optString("content_description", "").takeIf { it.isNotBlank() }
            "Barometer", "Magnetometer" -> data.optString("content_description", "").takeIf { it.isNotBlank() }
            "CodeView" -> data.optString("content_description", "").takeIf { it.isNotBlank() } ?: "code_view"
            else -> null
        }
    }

    private fun scheduleBindingUpdate(
        bindKey: String,
        value: String,
        actionName: String? = null,
        needsFilePicker: Boolean = false,
        allowMultiple: Boolean = false,
        delayMs: Long = 120L
    ) {
        pendingBindingUpdates.remove(bindKey)?.let { mainHandler.removeCallbacks(it) }
        val runnable = Runnable {
            bindings[bindKey] = value
            pendingBindingUpdates.remove(bindKey)
            if (!actionName.isNullOrEmpty()) {
                onAction(actionName, needsFilePicker, allowMultiple, bindings.toMap())
            }
        }
        pendingBindingUpdates[bindKey] = runnable
        mainHandler.postDelayed(runnable, delayMs)
    }

    private fun flushPendingBindings() {
        if (pendingBindingUpdates.isEmpty()) return
        val pending = pendingBindingUpdates.toMap()
        pendingBindingUpdates.clear()
        pending.values.forEach { runnable ->
            mainHandler.removeCallbacks(runnable)
            runnable.run()
        }
    }

    private fun matchesMeta(view: View, type: String, nodeId: String?): Boolean {
        val meta = view.getTag(renderMetaTag) as? RenderMeta ?: return false
        return meta.type == type && meta.nodeId == nodeId
    }

    private fun setMeta(view: View, type: String, nodeId: String?) {
        view.setTag(renderMetaTag, RenderMeta(type, nodeId))
    }

    private fun findReusableChild(parent: ViewGroup, data: JSONObject): View? {
        val nodeId = resolveNodeId(data) ?: return null
        val type = data.optString("type", "")
        for (i in 0 until parent.childCount) {
            val child = parent.getChildAt(i)
            val meta = child.getTag(renderMetaTag) as? RenderMeta
            if (meta != null && meta.nodeId == nodeId && meta.type == type) {
                return child
            }
            if (child is ViewGroup) {
                val nested = findReusableChild(child, data)
                if (nested != null) return nested
            }
        }
        return null
    }

    private fun detachFromParent(view: View, targetParent: ViewGroup) {
        val parent = view.parent as? ViewGroup
        if (parent != null && parent !== targetParent) {
            parent.removeView(view)
        }
    }

    private fun configureCodeWebView(webView: WebView, wrap: Boolean, contentDescription: String) {
        webView.settings.javaScriptEnabled = true
        webView.settings.cacheMode = WebSettings.LOAD_NO_CACHE
        webView.settings.setSupportZoom(false)
        webView.settings.displayZoomControls = false
        webView.settings.builtInZoomControls = false
        webView.settings.domStorageEnabled = false
        webView.settings.setSupportMultipleWindows(false)
        webView.isVerticalScrollBarEnabled = true
        webView.isHorizontalScrollBarEnabled = !wrap
        webView.setBackgroundColor(Color.TRANSPARENT)
        webView.contentDescription = contentDescription.takeIf { it.isNotEmpty() }
    }

    private fun computeColumns(data: JSONObject): Int {
        val explicit = data.optInt("columns", -1)
        if (explicit > 0) return explicit
        val screenWidthDp = context.resources.displayMetrics.widthPixels /
            context.resources.displayMetrics.density
        return if (screenWidthDp < 380) 1 else 2
    }

    private class CompassGLView(context: Context) : GLSurfaceView(context) {
        private val renderer = CompassRenderer()

        init {
            setEGLContextClientVersion(2)
            setRenderer(renderer)
            renderMode = RENDERMODE_WHEN_DIRTY
        }

        fun setAngle(angle: Float) {
            renderer.setAngle(angle)
            requestRender()
        }

        private class CompassRenderer : Renderer {
            private var program = 0
            private var resolutionHandle = 0
            private var angleHandle = 0
            private var width = 1
            private var height = 1
            private var angle = 0f

            override fun onSurfaceCreated(unused: GL10?, config: EGLConfig?) {
                val vertex = """
                    attribute vec4 a_position;
                    void main() { gl_Position = a_position; }
                """.trimIndent()
                program = createProgram(vertex, COMPASS_GLSL_FRAGMENT)
                GLES20.glUseProgram(program)
                angleHandle = GLES20.glGetUniformLocation(program, "u_angle")
                resolutionHandle = GLES20.glGetUniformLocation(program, "u_resolution")
            }

            override fun onSurfaceChanged(unused: GL10?, width: Int, height: Int) {
                this.width = width
                this.height = height
                GLES20.glViewport(0, 0, width, height)
                GLES20.glUniform2f(resolutionHandle, width.toFloat(), height.toFloat())
            }

            override fun onDrawFrame(unused: GL10?) {
                GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)
                GLES20.glUniform1f(angleHandle, angle)
                GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)
            }

            fun setAngle(value: Float) {
                angle = value
            }

            private fun createProgram(vs: String, fs: String): Int {
                val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vs)
                val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fs)
                val program = GLES20.glCreateProgram()
                GLES20.glAttachShader(program, vertexShader)
                GLES20.glAttachShader(program, fragmentShader)
                val positionHandle = 0
                GLES20.glBindAttribLocation(program, positionHandle, "a_position")
                GLES20.glLinkProgram(program)
                GLES20.glUseProgram(program)
                val verts = floatArrayOf(
                    -1f, -1f,
                    1f, -1f,
                    -1f, 1f,
                    1f, 1f
                )
                val buffer = java.nio.ByteBuffer.allocateDirect(verts.size * 4)
                    .order(java.nio.ByteOrder.nativeOrder())
                    .asFloatBuffer()
                buffer.put(verts).position(0)
                GLES20.glVertexAttribPointer(positionHandle, 2, GLES20.GL_FLOAT, false, 0, buffer)
                GLES20.glEnableVertexAttribArray(positionHandle)
                return program
            }

            private fun loadShader(type: Int, code: String): Int {
                val shader = GLES20.glCreateShader(type)
                GLES20.glShaderSource(shader, code)
                GLES20.glCompileShader(shader)
                return shader
            }
        }
    }

    private class SensorShaderView(
        context: Context,
        private val fragmentSrc: String,
        private val uniformName: String
    ) : GLSurfaceView(context) {
        private val renderer = SimpleValueRenderer(fragmentSrc, uniformName)

        init {
            setEGLContextClientVersion(2)
            setRenderer(renderer)
            renderMode = RENDERMODE_WHEN_DIRTY
        }

        fun setValue(v: Float) {
            renderer.setValue(v)
            requestRender()
        }

        private class SimpleValueRenderer(
            private val fragmentSrc: String,
            private val uniformName: String
        ) : Renderer {
            private var program = 0
            private var resolutionHandle = 0
            private var valueHandle = 0
            private var width = 1
            private var height = 1
            private var value = 0f

            override fun onSurfaceCreated(unused: GL10?, config: EGLConfig?) {
                val vertex = """
                    attribute vec4 a_position;
                    void main() { gl_Position = a_position; }
                """.trimIndent()
                program = createProgram(vertex, fragmentSrc)
                GLES20.glUseProgram(program)
                valueHandle = GLES20.glGetUniformLocation(program, uniformName)
                resolutionHandle = GLES20.glGetUniformLocation(program, "u_resolution")
            }

            override fun onSurfaceChanged(unused: GL10?, width: Int, height: Int) {
                this.width = width
                this.height = height
                GLES20.glViewport(0, 0, width, height)
                GLES20.glUniform2f(resolutionHandle, width.toFloat(), height.toFloat())
            }

            override fun onDrawFrame(unused: GL10?) {
                GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)
                GLES20.glUniform1f(valueHandle, value)
                GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)
            }

            fun setValue(v: Float) {
                value = v
            }

            private fun createProgram(vs: String, fs: String): Int {
                val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vs)
                val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fs)
                val program = GLES20.glCreateProgram()
                GLES20.glAttachShader(program, vertexShader)
                GLES20.glAttachShader(program, fragmentShader)
                val positionHandle = 0
                GLES20.glBindAttribLocation(program, positionHandle, "a_position")
                GLES20.glLinkProgram(program)
                GLES20.glUseProgram(program)
                val verts = floatArrayOf(
                    -1f, -1f,
                    1f, -1f,
                    -1f, 1f,
                    1f, 1f
                )
                val buffer = java.nio.ByteBuffer.allocateDirect(verts.size * 4)
                    .order(java.nio.ByteOrder.nativeOrder())
                    .asFloatBuffer()
                buffer.put(verts).position(0)
                GLES20.glVertexAttribPointer(positionHandle, 2, GLES20.GL_FLOAT, false, 0, buffer)
                GLES20.glEnableVertexAttribArray(positionHandle)
                return program
            }

            private fun loadShader(type: Int, code: String): Int {
                val shader = GLES20.glCreateShader(type)
                GLES20.glShaderSource(shader, code)
                GLES20.glCompileShader(shader)
                return shader
            }
        }
    }

    private class ShaderToyView(context: Context, fragmentSrc: String) : GLSurfaceView(context) {
        init {
            setEGLContextClientVersion(2)
            setRenderer(SimpleRenderer(fragmentSrc))
            renderMode = RENDERMODE_CONTINUOUSLY
        }

        private class SimpleRenderer(private val fragmentSrc: String) : Renderer {
            private var program = 0
            private var timeStart = 0L
            private var resolutionHandle = 0
            private var timeHandle = 0
            private var width = 1
            private var height = 1

            override fun onSurfaceCreated(unused: GL10?, config: EGLConfig?) {
                val vertex = """
                    attribute vec4 a_position;
                    void main() { gl_Position = a_position; }
                """.trimIndent()
                program = createProgram(vertex, fragmentSrc)
                GLES20.glUseProgram(program)
                timeHandle = GLES20.glGetUniformLocation(program, "u_time")
                resolutionHandle = GLES20.glGetUniformLocation(program, "u_resolution")
                timeStart = System.nanoTime()
            }

            override fun onSurfaceChanged(unused: GL10?, width: Int, height: Int) {
                this.width = width
                this.height = height
                GLES20.glViewport(0, 0, width, height)
            }

            override fun onDrawFrame(unused: GL10?) {
                GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)
                val t = (System.nanoTime() - timeStart) / 1_000_000_000.0f
                GLES20.glUniform1f(timeHandle, t)
                GLES20.glUniform2f(resolutionHandle, width.toFloat(), height.toFloat())
                GLES20.glDrawArrays(GLES20.GL_TRIANGLE_STRIP, 0, 4)
            }

            private fun createProgram(vs: String, fs: String): Int {
                val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vs)
                val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fs)
                val program = GLES20.glCreateProgram()
                GLES20.glAttachShader(program, vertexShader)
                GLES20.glAttachShader(program, fragmentShader)
                val positionHandle = 0
                GLES20.glBindAttribLocation(program, positionHandle, "a_position")
                GLES20.glLinkProgram(program)
                GLES20.glUseProgram(program)
                val verts = floatArrayOf(
                    -1f, -1f,
                    1f, -1f,
                    -1f, 1f,
                    1f, 1f
                )
                val buffer = java.nio.ByteBuffer.allocateDirect(verts.size * 4)
                    .order(java.nio.ByteOrder.nativeOrder())
                    .asFloatBuffer()
                buffer.put(verts).position(0)
                GLES20.glVertexAttribPointer(positionHandle, 2, GLES20.GL_FLOAT, false, 0, buffer)
                GLES20.glEnableVertexAttribArray(positionHandle)
                return program
            }

            private fun loadShader(type: Int, code: String): Int {
                val shader = GLES20.glCreateShader(type)
                GLES20.glShaderSource(shader, code)
                GLES20.glCompileShader(shader)
                return shader
            }
        }
    }

    companion object {
        private const val DEFAULT_FRAGMENT = """
            precision mediump float;
            uniform float u_time;
            uniform vec2 u_resolution;
            void main() {
                vec2 uv = gl_FragCoord.xy / u_resolution.xy;
                vec3 col = 0.5 + 0.5 * cos(u_time*0.2 + uv.xyx + vec3(0.0,2.0,4.0));
                gl_FragColor = vec4(col, 1.0);
            }
        """
        private const val COMPASS_GLSL_FRAGMENT = """
            precision mediump float;
            uniform vec2 u_resolution;
            uniform float u_angle;

            const float PI = 3.14159265359;
            const float TWO_PI = 6.28318530718;

            vec2 rotate(vec2 uv, float a) {
                float s = sin(a);
                float c = cos(a);
                return vec2(uv.x * c - uv.y * s, uv.x * s + uv.y * c);
            }

            float sdCircle(vec2 p, float r) {
                return length(p) - r;
            }

            float sdTriangle(vec2 p, float r) {
                const float k = 1.73205080757;
                p.x = abs(p.x) - r;
                p.y = p.y + r / k;
                if (p.x + k * p.y > 0.0) p = vec2(p.x - k * p.y, -k * p.x - p.y) / 2.0;
                p.x -= clamp(p.x, -2.0 * r, 0.0);
                return -length(p) * sign(p.y);
            }

            float drawTicks(vec2 p, float r, float numTicks, float thick, float len) {
                float angle = atan(p.y, p.x);
                float dist = length(p);
                float tickAngle = TWO_PI / numTicks;
                float a = mod(angle + (tickAngle/2.0), tickAngle) - (tickAngle/2.0);
                float mask = step(r - len, dist) * step(dist, r);
                float w = abs(a) * dist;
                return (1.0 - smoothstep(0.0, thick, w)) * mask;
            }

            void main() {
                vec2 fragCoord = gl_FragCoord.xy;
                vec2 uv = (fragCoord - 0.5 * u_resolution) / min(u_resolution.x, u_resolution.y);
                vec3 color = vec3(0.1, 0.12, 0.15);
                vec2 headerUV = uv;
                headerUV.y -= 0.38;
                float pointer = sdTriangle(vec2(headerUV.x, -headerUV.y), 0.03);
                float pointerFill = 1.0 - smoothstep(0.0, 0.005, pointer);
                vec2 rotUV = rotate(uv, u_angle);
                float rotLen = length(rotUV);
                float circleDist = abs(sdCircle(uv, 0.35));
                float circleRim = 1.0 - smoothstep(0.002, 0.008, circleDist);
                float minorTicks = drawTicks(rotUV, 0.33, 36.0, 0.005, 0.03);
                float majorTicks = drawTicks(rotUV, 0.33, 4.0, 0.015, 0.06);
                vec2 northUV = rotUV;
                northUV.y -= 0.25;
                float northMark = sdTriangle(vec2(northUV.x, -northUV.y), 0.04);
                float northFill = 1.0 - smoothstep(0.0, 0.005, northMark);
                color = mix(color, vec3(0.5, 0.8, 1.0), circleRim);
                color = mix(color, vec3(0.6, 0.6, 0.7), minorTicks);
                color = mix(color, vec3(1.0, 1.0, 1.0), majorTicks);
                color = mix(color, vec3(1.0, 0.2, 0.2), northFill);
                float southDist = sdCircle(rotUV + vec2(0.0, 0.25), 0.015);
                float southFill = 1.0 - smoothstep(0.0, 0.005, southDist);
                color = mix(color, vec3(0.2, 0.5, 1.0), southFill);
                color = mix(color, vec3(1.0, 0.7, 0.0), pointerFill);
                float glass = smoothstep(0.35, 0.0, rotLen) * 0.2;
                color += vec3(glass);
                gl_FragColor = vec4(color, 1.0);
            }
        """
        private const val BAROMETER_FRAGMENT = """
            precision mediump float;
            uniform vec2 u_resolution;
            uniform float u_value;
            void main() {
                vec2 uv = gl_FragCoord.xy / u_resolution;
                float t = clamp(u_value / 1100.0, 0.0, 1.0);
                vec3 base = mix(vec3(0.1,0.12,0.15), vec3(0.2,0.5,1.0), t);
                float ring = smoothstep(0.4, 0.38, length(uv - 0.5));
                float dot = smoothstep(0.03, 0.02, length(uv - vec2(0.5, 0.2 + 0.2 * t)));
                vec3 color = base;
                color = mix(color, vec3(0.8,0.9,1.0), ring);
                color = mix(color, vec3(1.0,0.7,0.2), dot);
                gl_FragColor = vec4(color, 1.0);
            }
        """
        private const val MAGNETOMETER_FRAGMENT = """
            precision mediump float;
            uniform vec2 u_resolution;
            uniform float u_value;
            void main() {
                vec2 uv = gl_FragCoord.xy / u_resolution;
                float m = clamp(u_value / 120.0, 0.0, 1.0);
                vec3 base = mix(vec3(0.08,0.1,0.14), vec3(1.0,0.2,0.2), m);
                float ring = smoothstep(0.35, 0.33, length(uv - 0.5));
                float glow = smoothstep(0.2, 0.0, abs(length(uv - 0.5) - 0.25));
                vec3 color = base;
                color += ring * vec3(0.6,0.6,0.9);
                color += glow * vec3(0.2,0.6,1.0);
                gl_FragColor = vec4(color, 1.0);
            }
        """
        private const val COMPASS_SHADER = """
            uniform float2 uResolution;
            uniform float uAngle;
            
            const float PI = 3.14159265359;
            const float TWO_PI = 6.28318530718;
            
            float2 rotate(float2 uv, float a) {
                float s = sin(a);
                float c = cos(a);
                return float2(uv.x * c - uv.y * s, uv.x * s + uv.y * c);
            }
            
            float sdCircle(float2 p, float r) {
                return length(p) - r;
            }
            
            float sdTriangle(float2 p, float r) {
                const float k = sqrt(3.0);
                p.x = abs(p.x) - r;
                p.y = p.y + r / k;
                if (p.x + k * p.y > 0.0) p = float2(p.x - k * p.y, -k * p.x - p.y) / 2.0;
                p.x -= clamp(p.x, -2.0 * r, 0.0);
                return -length(p) * sign(p.y);
            }
            
            float drawTicks(float2 p, float r, float numTicks, float thick, float len) {
                float angle = atan(p.y, p.x);
                float dist = length(p);
                float tickAngle = TWO_PI / numTicks;
                float a = mod(angle + (tickAngle/2.0), tickAngle) - (tickAngle/2.0);
                float2 tickPos = float2(cos(a), sin(a)) * dist;
                float d = abs(dist - r);
                float mask = step(r - len, dist) * step(dist, r);
                float w = abs(a) * dist;
                return (1.0 - smoothstep(0.0, thick, w)) * mask;
            }
            
            half4 main(float2 fragCoord) {
                float2 uv = (fragCoord - 0.5 * uResolution) / min(uResolution.x, uResolution.y);
                half3 color = half3(0.1, 0.12, 0.15);
                float2 headerUV = uv;
                headerUV.y -= 0.38;
                float pointer = sdTriangle(float2(headerUV.x, -headerUV.y), 0.03);
                float pointerFill = 1.0 - smoothstep(0.0, 0.005, pointer);
                float2 rotUV = rotate(uv, uAngle);
                float rotLen = length(rotUV);
                float circleDist = abs(sdCircle(uv, 0.35));
                float circleRim = 1.0 - smoothstep(0.002, 0.008, circleDist);
                float minorTicks = drawTicks(rotUV, 0.33, 36.0, 0.005, 0.03);
                float majorTicks = drawTicks(rotUV, 0.33, 4.0, 0.015, 0.06);
                float2 northUV = rotUV;
                northUV.y -= 0.25;
                float northMark = sdTriangle(float2(northUV.x, -northUV.y), 0.04);
                float northFill = 1.0 - smoothstep(0.0, 0.005, northMark);
                color = mix(color, half3(0.5, 0.8, 1.0), circleRim);
                color = mix(color, half3(0.6, 0.6, 0.7), minorTicks);
                color = mix(color, half3(1.0, 1.0, 1.0), majorTicks);
                color = mix(color, half3(1.0, 0.2, 0.2), northFill);
                float southDist = sdCircle(rotUV + float2(0.0, 0.25), 0.015);
                float southFill = 1.0 - smoothstep(0.0, 0.005, southDist);
                color = mix(color, half3(0.2, 0.5, 1.0), southFill);
                color = mix(color, half3(1.0, 0.7, 0.0), pointerFill);
                float glass = smoothstep(0.35, 0.0, rotLen) * 0.2;
                color += half3(glass);
                return half4(color, 1.0);
            }
        """
    }

    private fun escapeHtml(text: String): String {
        return text
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
    }

    private fun dpToPx(context: Context, dp: Float): Int {
        val density = context.resources.displayMetrics.density
        return (dp * density).toInt()
    }

    private fun copyToClipboard(text: String) {
        val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager ?: return
        cm.setPrimaryClip(ClipData.newPlainText("copy", text))
    }
}
