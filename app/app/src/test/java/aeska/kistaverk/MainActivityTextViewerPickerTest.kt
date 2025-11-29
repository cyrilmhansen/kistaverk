package aeska.kistaverk

import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.io.File

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33], shadows = [ShadowSystemLoadLibrary::class, ShadowMainActivity::class])
class MainActivityTextViewerPickerTest {

    @Before
    fun setup() {
        System.setProperty("kistaverk.skipNativeLoad", "true")
        ShadowMainActivity.reset()
    }

    @After
    fun tearDown() {
        ShadowMainActivity.reset()
    }

    @Test
    fun textViewerPickerMapsScreenActionToOpen() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        val tempFile = File(context.cacheDir, "sample.txt").apply {
            writeText("hello world")
        }
        val uri = Uri.fromFile(tempFile)

        val controller = Robolectric.buildActivity(MainActivity::class.java).setup()
        val activity = controller.get()

        val handled = activity.handlePickerResultForTest("text_viewer_screen", uri, emptyMap())
        assertTrue("Picker result should be handled", handled)
        assertTrue(
            "Picker should dispatch text_viewer_open after selection",
            ShadowMainActivity.actions.contains("text_viewer_open")
        )

        controller.pause().stop().destroy()
    }
}
