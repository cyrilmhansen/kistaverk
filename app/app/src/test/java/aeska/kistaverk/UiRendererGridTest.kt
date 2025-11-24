package aeska.kistaverk

import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UiRendererGridTest {

    @Test
    fun grid_lays_out_children_in_rows() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                {
                  "type": "Grid",
                  "columns": 2,
                  "children": [
                    { "type": "Button", "text": "One", "action": "a" },
                    { "type": "Button", "text": "Two", "action": "b" },
                    { "type": "Button", "text": "Three", "action": "c" }
                  ]
                }
              ]
            }
        """.trimIndent()

        val view = renderer.render(ui)
        val root = (view as ScrollView).getChildAt(0) as LinearLayout
        val grid = root.getChildAt(0) as LinearLayout
        val firstRow = grid.getChildAt(0) as LinearLayout
        val secondRow = grid.getChildAt(1) as LinearLayout

        val firstRowButton = firstRow.getChildAt(0) as Button
        val secondRowButton = secondRow.getChildAt(0) as Button

        assertEquals("One", firstRowButton.text)
        assertEquals("Three", secondRowButton.text)
    }
}
