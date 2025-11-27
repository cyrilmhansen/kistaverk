package aeska.kistaverk

import android.webkit.WebView
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
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
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
        val view = renderer.render(ui) as ScrollView
        val webView = view.getChildAt(0) as WebView
        val url = webView.url ?: ""
        assertTrue(url.contains("android_asset/prism/"))
    }

    @Test
    fun codeViewMissingTextFailsValidation() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """{ "type": "CodeView", "language": "rust" }"""
        val view = renderer.render(ui) as ScrollView
        val title = view.getChildAt(0) as TextView
        val msg = view.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("CodeView missing text"))
    }
}
