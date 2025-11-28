package aeska.kistaverk

import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.shadows.ShadowAssetManager

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class UiRendererDepsListTest {

    @Test
    fun depsListRendersEntriesWhenAssetPresent() {
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { _, _, _ -> }
        val ui = """{ "type": "Column", "children": [ { "type": "DepsList" } ] }""".trimIndent()

        val root = TestViews.unwrap(renderer.render(ui)) as? ScrollView ?: error("Expected ScrollView root")
        val column = root.getChildAt(0) as? LinearLayout ?: error("Expected Column child")
        val depsScroll = column.getChildAt(0) as? ScrollView ?: error("Expected ScrollView deps list")
        val inner = depsScroll.getChildAt(0) as? LinearLayout ?: error("Expected deps list inner layout")
        // Expect at least one entry rendered from generated deps.json
        val hasEntries = inner.childCount > 0
        assertTrue(hasEntries)
    }
}
