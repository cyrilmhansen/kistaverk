package aeska.kistaverk

import android.view.View
import android.view.inputmethod.EditorInfo
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

@RunWith(RobolectricTestRunner::class)
class UiRendererTextInputTest {

    private fun render(json: String, actions: MutableList<Triple<String, Boolean, Map<String, String>>>): View {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, picker, bindings ->
            actions.add(Triple(action, picker, bindings))
        }
        return TestViews.unwrap(renderer.render(json))
    }

    @Test
    fun buttonClick_sendsCurrentBindings_fromTextInput() {
        val actions = mutableListOf<Triple<String, Boolean, Map<String, String>>>()
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "TextInput", "bind_key": "text_input", "text": "hello", "hint": "enter" },
                { "type": "Button", "text": "Submit", "action": "text_tools_upper" }
              ]
            }
        """.trimIndent()

        val view = render(ui, actions)
        val rootLayout = (view as ScrollView).getChildAt(0) as LinearLayout
        val editText = rootLayout.getChildAt(0) as EditText
        val button = rootLayout.getChildAt(1) as Button

        // Simulate user typing
        editText.setText("hello world")
        button.performClick()

        val (action, needsPicker, bindings) = actions.last()
        assertEquals("text_tools_upper", action)
        assertEquals(false, needsPicker)
        assertEquals("hello world", bindings["text_input"])
    }

    @Test
    fun imeDone_triggersActionOnSubmit_withLatestText() {
        val actions = mutableListOf<Triple<String, Boolean, Map<String, String>>>()
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "TextInput", "bind_key": "text_input", "action_on_submit": "text_tools_word_count" },
                { "type": "Text", "text": "helper" }
              ]
            }
        """.trimIndent()

        val view = render(ui, actions)
        val rootLayout = (view as ScrollView).getChildAt(0) as LinearLayout
        val editText = rootLayout.getChildAt(0) as EditText

        editText.setText("one two three")

        // Trigger IME action
        val before = actions.size
        editText.onEditorAction(EditorInfo.IME_ACTION_DONE)
        assertEquals(before + 1, actions.size)

        val (action, needsPicker, bindings) = actions.last()
        assertEquals("text_tools_word_count", action)
        assertEquals(false, needsPicker)
        assertEquals("one two three", bindings["text_input"])
    }
}
