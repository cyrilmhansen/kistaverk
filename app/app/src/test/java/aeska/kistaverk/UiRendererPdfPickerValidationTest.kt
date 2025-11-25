package aeska.kistaverk

import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class UiRendererPdfPickerValidationTest {

    private fun render(ui: String): Pair<TextView, TextView> {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val view = renderer.render(ui)
        val root = view as ScrollView
        val column = root.getChildAt(0) as LinearLayout
        val title = column.getChildAt(0) as TextView
        val msg = column.getChildAt(1) as TextView
        return title to msg
    }

    @Test
    fun pdfPickerMissingSourceUriFailsValidation() {
        val ui = """{ "type": "PdfPagePicker", "page_count": 3, "bind_key": "k" }"""
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("PdfPagePicker missing source_uri"))
    }

    @Test
    fun pdfPickerMissingBindKeyFailsValidation() {
        val ui = """{ "type": "PdfPagePicker", "page_count": 3, "source_uri": "x" }"""
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("PdfPagePicker missing bind_key"))
    }

    @Test
    fun pdfPickerMissingPageCountFailsValidation() {
        val ui = """{ "type": "PdfPagePicker", "bind_key": "k", "source_uri": "x" }"""
        val (title, msg) = render(ui)
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("PdfPagePicker missing page_count"))
    }
}
