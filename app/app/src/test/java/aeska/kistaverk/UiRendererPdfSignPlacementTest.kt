package aeska.kistaverk

import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
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
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, _, bindings ->
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

        // Simulate a tap near bottom-right to check clamping to <=1.0
        overlay.dispatchTouchEvent(android.view.MotionEvent.obtain(0, 0, android.view.MotionEvent.ACTION_DOWN, overlay.width * 0.9f, overlay.height * 0.9f, 0))
        overlay.dispatchTouchEvent(android.view.MotionEvent.obtain(0, 0, android.view.MotionEvent.ACTION_UP, overlay.width * 0.9f, overlay.height * 0.9f, 0))

        assertTrue(recorded.isNotEmpty())
        val (_, bindings) = recorded.last()
        assertEquals("2", bindings["pdf_signature_page"])
        val x = bindings["pdf_signature_x_pct"]?.toFloat() ?: 0f
        val y = bindings["pdf_signature_y_pct"]?.toFloat() ?: 0f
        assertTrue(x in 0f..1f)
        assertTrue(y in 0f..1f)
    }

    @Test
    fun pagingButtonsUpdatePageBinding() {
        val recorded = mutableListOf<Pair<String, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, _, bindings ->
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

        val (_, bindings) = recorded.last()
        assertEquals("2", bindings["pdf_signature_page"])
    }
}
