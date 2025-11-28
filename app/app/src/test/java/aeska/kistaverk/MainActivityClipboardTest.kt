package aeska.kistaverk

import android.content.ClipData
import android.content.ClipboardManager
import android.widget.Button
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33], shadows = [ShadowSystemLoadLibrary::class, ShadowMainActivity::class])
class MainActivityClipboardTest {

    @Test
    fun copyTextButton_writesClipboard() {
        val actions = mutableListOf<Triple<String, Boolean, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, picker, bindings ->
            actions.add(Triple(action, picker, bindings))
        }
        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Button", "text": "Copy", "copy_text": "#AABBCC" }
              ]
            }
        """.trimIndent()
        val root = TestViews.unwrap(renderer.render(ui)) as android.widget.ScrollView
        val btn = root.getChildAt(0) as android.widget.LinearLayout
        (btn.getChildAt(0) as Button).performClick()

        val cm = ApplicationProvider.getApplicationContext<android.content.Context>()
            .getSystemService(android.content.Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clip = cm.primaryClip
        assertNotNull(clip)
        val text = clip?.getItemAt(0)?.coerceToText(ApplicationProvider.getApplicationContext())?.toString()
        assertEquals("#AABBCC", text)
    }

    @Test
    fun readHexFromClipboard_returnsNormalizedHex() {
        System.setProperty("kistaverk.skipNativeLoad", "true")
        val controller = Robolectric.buildActivity(MainActivity::class.java).setup()
        val activity = controller.get()
        val cm = activity.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as ClipboardManager
        cm.setPrimaryClip(ClipData.newPlainText("hex", "A1B2C3"))

        val method = MainActivity::class.java.getDeclaredMethod("readHexFromClipboard")
        method.isAccessible = true
        val result = method.invoke(activity) as? String
        assertEquals("#A1B2C3", result)
        controller.pause().stop().destroy()
    }
}
