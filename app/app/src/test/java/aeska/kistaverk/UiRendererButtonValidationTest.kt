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
class UiRendererButtonValidationTest {

    @Test
    fun button_missing_action_and_copytext_renders_error() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Button", "text": "No action" }
              ]
            }
        """.trimIndent()

        val root = (TestViews.unwrap(renderer.render(ui)) as ScrollView).getChildAt(0) as LinearLayout
        val title = root.getChildAt(0) as TextView
        val msg = root.getChildAt(1) as TextView
        assertTrue(title.text.toString().contains("Render error"))
        assertTrue(msg.text.toString().contains("Button missing action or copy_text"))
    }
}
