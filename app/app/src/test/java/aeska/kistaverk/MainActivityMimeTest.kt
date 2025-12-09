package aeska.kistaverk

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33], shadows = [ShadowSystemLoadLibrary::class, ShadowMainActivity::class])
class MainActivityMimeTest {

    @Test
    fun guessMimeFromPath_detectsGzip() {
        System.setProperty("kistaverk.skipNativeLoad", "true")
        val controller = Robolectric.buildActivity(MainActivity::class.java).setup()
        val activity = controller.get()

        val method = MainActivity::class.java.getDeclaredMethod("guessMimeFromPath", String::class.java)
        method.isAccessible = true

        val mime = method.invoke(activity, "/tmp/output.gz") as? String
        assertEquals("application/gzip", mime)

        val unknown = method.invoke(activity, "/tmp/file.unknown") as? String
        assertNull(unknown)

        controller.pause().stop().destroy()
    }
}
