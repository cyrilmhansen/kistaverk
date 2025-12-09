package aeska.kistaverk

import android.graphics.pdf.PdfDocument
import android.net.Uri
import android.widget.GridLayout
import android.widget.ImageView
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.io.File
import java.io.FileOutputStream

@RunWith(RobolectricTestRunner::class)
class UiRendererPdfPreviewTest {

    @Test
    fun pdf_preview_grid_renders_cells() {
        val ctx = ApplicationProvider.getApplicationContext<android.content.Context>()
        val file = createTestPdf(ctx.cacheDir, "preview_grid.pdf", 2)
        val renderer = UiRenderer(ctx) { _, _, _, _ -> }
        val ui = """
            {
              "type": "PdfPreviewGrid",
              "source_uri": "${file.toURI()}",
              "page_count": 2,
              "action": "pdf_page_open"
            }
        """.trimIndent()

        val root = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val grid = root.getChildAt(0) as GridLayout
        assertEquals(2, grid.childCount)
    }

    @Test
    fun pdf_single_page_renders_image() {
        val ctx = ApplicationProvider.getApplicationContext<android.content.Context>()
        val file = createTestPdf(ctx.cacheDir, "preview_single.pdf", 1)
        val renderer = UiRenderer(ctx) { _, _, _, _ -> }
        val ui = """
            {
              "type": "PdfSinglePage",
              "source_uri": "${file.toURI()}",
              "page": 1
            }
        """.trimIndent()

        val root = TestViews.unwrap(renderer.render(ui))
        val image = when (root) {
            is ScrollView -> root.getChildAt(0) as ImageView
            is ImageView -> root
            else -> throw AssertionError("Expected ImageView inside ScrollView")
        }
        assertNotNull(image.drawable)
    }

    private fun createTestPdf(dir: File, name: String, pages: Int): File {
        val file = File(dir, name)
        val doc = PdfDocument()
        repeat(pages) { index ->
            val pageInfo = PdfDocument.PageInfo.Builder(300, 400, index + 1).create()
            val page = doc.startPage(pageInfo)
            page.canvas.drawText("Page ${index + 1}", 50f, 200f, android.graphics.Paint())
            doc.finishPage(page)
        }
        FileOutputStream(file).use { out -> doc.writeTo(out) }
        doc.close()
        return file
    }
}
