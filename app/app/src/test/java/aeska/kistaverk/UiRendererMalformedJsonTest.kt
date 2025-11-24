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
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "MysteryWidget", "text": "Boom" }
              ]
            }
        """.trimIndent()

        val view = renderer.render(ui)
        val root = (view as ScrollView).getChildAt(0) as LinearLayout
        val error = root.getChildAt(0) as TextView
        assertTrue(error.text.toString().contains("Unknown"))
    }

    @Test
    fun missing_children_in_column_renders_error() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """{ "type": "Column" }"""

        val view = renderer.render(ui)
        val root = (view as ScrollView).getChildAt(0) as LinearLayout
        val error = root.getChildAt(0) as TextView
        assertTrue(error.text.toString().contains("Missing children"))
    }
}
