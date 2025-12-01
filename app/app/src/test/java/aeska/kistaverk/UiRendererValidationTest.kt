package aeska.kistaverk

import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UiRendererValidationTest {

    private fun render(json: String): Pair<TextView, TextView> {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val view = TestViews.unwrap(renderer.render(json)) as ScrollView
        val root = view.getChildAt(0) as LinearLayout
        val title = root.getChildAt(0) as TextView
        val msg = root.getChildAt(1) as TextView
        return title to msg
    }

    @Test
    fun button_without_action_or_copytext_fails_validation() {
        val ui = """
            { "type": "Column", "children": [ { "type": "Button", "text": "oops" } ] }
        """.trimIndent()
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("Button missing action or copy_text"))
    }

    @Test
    fun image_base64_missing_data_fails_validation() {
        val ui = """
            { "type": "ImageBase64" }
        """.trimIndent()
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("ImageBase64 missing base64"))
    }

    @Test
    fun color_swatch_missing_color_fails_validation() {
        val ui = """
            { "type": "ColorSwatch" }
        """.trimIndent()
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("ColorSwatch missing color"))
    }

    @Test
    fun pdf_preview_grid_requires_source_and_action() {
        val ui = """{ "type": "PdfPreviewGrid", "page_count": 2 }"""
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("PdfPreviewGrid missing source_uri"))
    }
}
