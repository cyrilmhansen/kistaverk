package aeska.kistaverk

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
class UiRendererSectionCardTest {

    private fun render(json: String) = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        .render(json)
        .let { TestViews.unwrap(it) as ScrollView }

    @Test
    fun section_missing_children_fails_validation() {
        val json = """{ "type": "Section", "title": "Empty" }"""
        val root = render(json)
        val layout = root.getChildAt(0) as LinearLayout
        val title = layout.getChildAt(0) as TextView
        val msg = layout.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("Section missing children"))
    }

    @Test
    fun card_missing_children_fails_validation() {
        val json = """{ "type": "Card", "title": "Empty" }"""
        val root = render(json)
        val layout = root.getChildAt(0) as LinearLayout
        val title = layout.getChildAt(0) as TextView
        val msg = layout.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("Card missing children"))
    }

    @Test
    fun section_and_card_render_headers_and_children() {
        val json = """
            {
              "type": "Column",
              "children": [
                {
                  "type": "Section",
                  "title": "📁 Files",
                  "subtitle": "2 tools",
                  "icon": "📁",
                  "children": [
                    { "type": "Text", "text": "Inside section" }
                  ]
                },
                {
                  "type": "Card",
                  "title": "Quick",
                  "children": [
                    { "type": "Text", "text": "Inside card" }
                  ]
                }
              ]
            }
        """.trimIndent()

        val scroll = render(json)
        val column = scroll.getChildAt(0) as LinearLayout
        val section = column.getChildAt(0) as LinearLayout
        val card = column.getChildAt(1) as LinearLayout

        // Section header: icon + title/subtitle column
        val sectionHeader = section.getChildAt(0) as LinearLayout
        val sectionTextCol = sectionHeader.getChildAt(1) as LinearLayout
        val sectionTitle = sectionTextCol.getChildAt(0) as TextView
        val sectionSubtitle = sectionTextCol.getChildAt(1) as TextView
        assertEquals("📁 Files", sectionTitle.text.toString())
        assertEquals("2 tools", sectionSubtitle.text.toString())
        val sectionBody = section.getChildAt(1) as TextView
        assertEquals("Inside section", sectionBody.text.toString())

        val cardHeader = card.getChildAt(0) as LinearLayout
        val cardTitle = (cardHeader.getChildAt(0) as LinearLayout).getChildAt(0) as TextView
        assertEquals("Quick", cardTitle.text.toString())
        val cardBody = card.getChildAt(1) as TextView
        assertEquals("Inside card", cardBody.text.toString())
    }
}
