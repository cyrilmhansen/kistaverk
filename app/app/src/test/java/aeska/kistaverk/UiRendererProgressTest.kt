package aeska.kistaverk

import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UiRendererProgressTest {

    @Test
    fun progress_widget_renders_and_is_accessible() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Progress", "text": "Working...", "content_description": "In progress" }
              ]
            }
        """.trimIndent()

        val view = renderer.render(ui)
        val root = (view as ScrollView).getChildAt(0) as LinearLayout
        val progressContainer = root.getChildAt(0) as LinearLayout
        val progress = progressContainer.getChildAt(1) as ProgressBar

        assertEquals("In progress", progressContainer.contentDescription)
        assertTrue(progress.isIndeterminate)
    }
}
