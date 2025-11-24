package aeska.kistaverk

import android.content.Context
import android.graphics.Color
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.view.View
import android.view.inputmethod.EditorInfo
import android.widget.Button
import android.widget.EditText
import android.widget.CheckBox
import android.widget.LinearLayout.LayoutParams
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.text.Editable
import android.text.TextWatcher
import android.widget.ProgressBar
import org.json.JSONObject
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

// Added 'onAction' callback: (String, Boolean) -> Unit where the boolean flags file picker needs
class UiRenderer(
    private val context: Context,
    private val onAction: (String, Boolean, Map<String, String>) -> Unit
) {
    private val bindings = mutableMapOf<String, String>()

    fun render(jsonString: String): View {
        bindings.clear()
        val root = createView(JSONObject(jsonString))
        return if (root is LinearLayout) {
            ScrollView(context).apply {
                layoutParams = FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.MATCH_PARENT,
                    FrameLayout.LayoutParams.MATCH_PARENT
                )
                addView(root)
            }
        } else {
            root
        }
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
            setOnClickListener { onAction("reset", false, emptyMap()) }
        })

        return layout
    }

    private fun createView(data: JSONObject): View {
        val type = data.optString("type")
        return when (type) {
            "Column" -> createColumn(data)
            "Text" -> createText(data)
            "Button" -> createButton(data)
            "ShaderToy" -> createShaderToy(data)
            "TextInput" -> createTextInput(data)
            "Checkbox" -> createCheckbox(data)
            "Progress" -> createProgress(data)
            "Grid" -> createGrid(data)
            else -> createErrorView("Unknown: $type")
        }
    }

    // WARNING: For createColumn, make sure to call createView recursively
    // I'm putting the abbreviated code back for clarity:
    private fun createColumn(data: JSONObject): View {
        val layout = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        if (data.has("padding")) {
            val p = data.getInt("padding")
            layout.setPadding(p, p, p, p)
        }
        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            layout.contentDescription = contentDescription
        }
        val children = data.optJSONArray("children")
        if (children != null) {
            for (i in 0 until children.length()) {
                layout.addView(createView(children.getJSONObject(i)))
            }
        }
        return layout
    }

    private fun createText(data: JSONObject): View {
        return TextView(context).apply {
            text = data.optString("text")
            textSize = data.optDouble("size", 14.0).toFloat()
            val contentDescription = data.optString("content_description", "")
            if (contentDescription.isNotEmpty()) {
                this.contentDescription = contentDescription
            }
        }
    }

    private fun createButton(data: JSONObject): View {
        val btn = Button(context)
        btn.text = data.optString("text")
        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            btn.contentDescription = contentDescription
        }

        // Retrieve the action defined in the Rust JSON (e.g., "hash_file")
        val actionName = data.optString("action")
        val needsFilePicker = data.optBoolean("requires_file_picker", false)

        btn.setOnClickListener {
            onAction(actionName, needsFilePicker, bindings.toMap())
        }
        return btn
    }

    private fun createErrorView(msg: String): View {
        return TextView(context).apply {
            text = msg
            setTextColor(Color.RED)
        }
    }

    private fun createTextInput(data: JSONObject): View {
        val editText = EditText(context)
        val bindKey = data.optString("bind_key", "")
        val initial = data.optString("text", "")
        editText.setText(initial)
        if (initial.isNotEmpty() && bindKey.isNotEmpty()) {
            bindings[bindKey] = initial
        }
        val hint = data.optString("hint", "")
        if (hint.isNotEmpty()) {
            editText.hint = hint
        }
        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            editText.contentDescription = contentDescription
        }

        val singleLine = data.optBoolean("single_line", false)
        editText.isSingleLine = singleLine
        val maxLines = data.optInt("max_lines", 0)
        if (maxLines > 0) {
            editText.maxLines = maxLines
        }

        if (bindKey.isNotEmpty()) {
            editText.addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) {
                    bindings[bindKey] = s?.toString().orEmpty()
                }

                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
            })
        }

        val submitAction = data.optString("action_on_submit", "")
        if (submitAction.isNotEmpty()) {
            editText.setOnEditorActionListener { _, actionId, _ ->
                val isDone = actionId == EditorInfo.IME_ACTION_DONE || actionId == EditorInfo.IME_NULL
                if (isDone) {
                    onAction(submitAction, false, bindings.toMap())
                }
                isDone
            }
        }
        return editText
    }

    private fun createShaderToy(data: JSONObject): View {
        val fragment = data.optString("fragment", DEFAULT_FRAGMENT)
        val view = ShaderToyView(context, fragment)
        val lp = LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.MATCH_PARENT,
            dpToPx(context, 240f)
        )
        val margin = dpToPx(context, 12f)
        lp.topMargin = margin
        lp.bottomMargin = margin
        view.layoutParams = lp
        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            view.contentDescription = contentDescription
        }
        return view
    }

    private fun createProgress(data: JSONObject): View {
        val container = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LayoutParams(
                LayoutParams.MATCH_PARENT,
                LayoutParams.WRAP_CONTENT
            )
        }
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
        if (contentDescription.isNotEmpty()) {
            container.contentDescription = contentDescription
        }
        return container
    }

    private fun createCheckbox(data: JSONObject): View {
        val checkBox = CheckBox(context)
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
                if (actionName.isNotEmpty()) {
                    onAction(actionName, needsFilePicker, bindings.toMap())
                }
            }
        }
        return checkBox
    }

    private fun createGrid(data: JSONObject): View {
        val columns = computeColumns(data)
        val children = data.optJSONArray("children") ?: return createErrorView("Grid missing children")
        val wrapper = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
        }
        val padding = data.optInt("padding", 0)
        if (padding > 0) {
            wrapper.setPadding(padding, padding, padding, padding)
        }
        val contentDescription = data.optString("content_description", "")
        if (contentDescription.isNotEmpty()) {
            wrapper.contentDescription = contentDescription
        }

        var row: LinearLayout? = null
        for (i in 0 until children.length()) {
            val childView = createView(children.getJSONObject(i))
            if (i % columns == 0) {
                row = LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
                }
                wrapper.addView(row)
            }
            val lp = LinearLayout.LayoutParams(0, LayoutParams.WRAP_CONTENT, 1f)
            childView.layoutParams = lp
            row?.addView(childView)
        }
        return wrapper
    }

    private fun computeColumns(data: JSONObject): Int {
        val explicit = data.optInt("columns", -1)
        if (explicit > 0) return explicit
        val screenWidthDp = context.resources.displayMetrics.widthPixels /
            context.resources.displayMetrics.density
        return if (screenWidthDp < 380) 1 else 2
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

        private fun dpToPx(context: Context, dp: Float): Int {
            val density = context.resources.displayMetrics.density
            return (dp * density).toInt()
        }
    }
}
