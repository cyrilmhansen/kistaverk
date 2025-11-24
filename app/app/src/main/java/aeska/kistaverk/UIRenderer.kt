package aeska.kistaverk

import android.content.Context
import android.graphics.Color
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.view.View
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import org.json.JSONObject
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.min

// Added 'onAction' callback: (String, Boolean) -> Unit where the boolean flags file picker needs
class UiRenderer(
    private val context: Context,
    private val onAction: (String, Boolean) -> Unit
) {

    fun render(jsonString: String): View {
        return createView(JSONObject(jsonString))
    }

    private fun createView(data: JSONObject): View {
        val type = data.optString("type")
        return when (type) {
            "Column" -> createColumn(data)
            "Text" -> createText(data)
            "Button" -> createButton(data)
            "ShaderToy" -> createShaderToy(data)
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
        }
    }

    private fun createButton(data: JSONObject): View {
        val btn = Button(context)
        btn.text = data.optString("text")

        // Retrieve the action defined in the Rust JSON (e.g., "hash_file")
        val actionName = data.optString("action")
        val needsFilePicker = data.optBoolean("requires_file_picker", false)

        btn.setOnClickListener {
            onAction(actionName, needsFilePicker)
        }
        return btn
    }

    private fun createErrorView(msg: String): View {
        return TextView(context).apply {
            text = msg
            setTextColor(Color.RED)
        }
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
        return view
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
