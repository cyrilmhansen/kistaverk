package aeska.kistaverk

import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class UiRendererBackButtonTest {

    @Test
    fun backButtonAppearsWhenDeclared() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Text", "text": "Nested" },
                { "type": "Button", "text": "Back", "action": "back" }
              ]
            }
        """.trimIndent()

        val root = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val column = root.getChildAt(0) as LinearLayout
        val hasBack = (0 until column.childCount).any { idx ->
            (column.getChildAt(idx) as? Button)?.text?.toString() == "Back"
        }
        assertTrue(hasBack)
    }
}
