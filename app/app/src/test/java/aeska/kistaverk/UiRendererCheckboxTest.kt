package aeska.kistaverk

import android.widget.CheckBox
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UiRendererCheckboxTest {

    @Test
    fun checkboxToggle_updatesBindings_andTriggersAction() {
        val actions = mutableListOf<Triple<String, Boolean, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, picker, bindings ->
            actions.add(Triple(action, picker, bindings))
        }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Checkbox", "bind_key": "aggressive_trim", "checked": true, "action": "text_tools_refresh" },
                { "type": "Button", "text": "noop", "action": "noop" }
              ]
            }
        """.trimIndent()

        val view = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val rootLayout = view.getChildAt(0) as LinearLayout
        val checkbox = rootLayout.getChildAt(0) as CheckBox

        // Toggle off
        checkbox.isChecked = false

        val (action, needsPicker, bindings) = actions.last()
        assertEquals("text_tools_refresh", action)
        assertEquals(false, needsPicker)
        assertEquals("false", bindings["aggressive_trim"])
    }
}
