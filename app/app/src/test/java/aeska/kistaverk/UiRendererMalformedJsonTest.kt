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
class UiRendererMalformedJsonTest {

    @Test
    fun unknown_type_renders_error_view() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "MysteryWidget", "text": "Boom" }
              ]
            }
        """.trimIndent()

        val view = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val root = view.getChildAt(0) as LinearLayout
        val title = root.getChildAt(0) as TextView
        val msg = root.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("Unknown widget"))
    }

    @Test
    fun missing_children_in_column_renders_error() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _, _ -> }
        val ui = """{ "type": "Column" }"""

        val view = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val root = view.getChildAt(0) as LinearLayout
        val title = root.getChildAt(0) as TextView
        val msg = root.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("missing children", ignoreCase = true))
    }
}
