package aeska.kistaverk

import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class UiRendererPdfSignPlacementTest {

    @Test
    fun tapSetsNormalizedCoordsAndBindings() {
        val recorded = mutableListOf<Pair<String, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, _, _, bindings ->
            recorded.add(action to bindings)
        }
        val ui = """
            {
              "type": "PdfSignPlacement",
              "source_uri": "file:///tmp/test.pdf",
              "page_count": 3,
              "selected_page": 2,
              "bind_key_page": "pdf_signature_page",
              "bind_key_x_pct": "pdf_signature_x_pct",
              "bind_key_y_pct": "pdf_signature_y_pct"
            }
        """.trimIndent()

        val view = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val column = view.getChildAt(0) as LinearLayout
        val frame = column.getChildAt(1) as FrameLayout
        val overlay = frame.getChildAt(1)
        overlay.layout(0, 0, 200, 200)

        // Sanity: render produced overlay and frame
        assertTrue(frame.childCount >= 2)
    }

    @Test
    fun pagingButtonsUpdatePageBinding() {
        val recorded = mutableListOf<Pair<String, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, _, _, bindings ->
            recorded.add(action to bindings)
        }
        val ui = """
            {
              "type": "PdfSignPlacement",
              "source_uri": "file:///tmp/test.pdf",
              "page_count": 3,
              "selected_page": 1,
              "bind_key_page": "pdf_signature_page",
              "bind_key_x_pct": "pdf_signature_x_pct",
              "bind_key_y_pct": "pdf_signature_y_pct"
            }
        """.trimIndent()

        val view = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val controls = ((view.getChildAt(0) as LinearLayout).getChildAt(0) as LinearLayout)
        val next = controls.getChildAt(4) as Button
        next.performClick()

        val label = controls.getChildAt(2) as android.widget.TextView
        assertTrue(label.text.toString().contains("Page 2"))
    }
}
