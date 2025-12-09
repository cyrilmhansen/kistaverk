package aeska.kistaverk

import android.view.ViewGroup
import android.webkit.WebView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class UiRendererCodeViewTest {

    @Test
    fun codeViewRendersHtmlBase() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _, _ -> }
        val ui = """
            {
              "type": "CodeView",
              "text": "fn main() { println!(\"hi\"); }",
              "language": "rust",
              "line_numbers": true,
              "wrap": true,
              "theme": "dark"
            }
        """.trimIndent()
        val view = TestViews.unwrap(renderer.render(ui))
        val webView = findWebView(view)
        assertTrue(webView != null)
    }

    @Test
    fun codeViewMissingTextFailsValidation() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _, _ -> }
        val ui = """{ "type": "CodeView", "language": "rust" }"""
        val scroll = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val root = scroll.getChildAt(0) as LinearLayout
        val title = root.getChildAt(0) as TextView
        val msg = root.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("CodeView missing text"))
    }

    private fun findWebView(root: android.view.View): WebView? {
        if (root is WebView) return root
        if (root is ViewGroup) {
            for (i in 0 until root.childCount) {
                findWebView(root.getChildAt(i))?.let { return it }
            }
        }
        return null
    }
}
