package aeska.kistaverk

import android.os.Bundle
import androidx.test.core.app.ApplicationProvider
import org.json.JSONObject
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.Robolectric

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33], shadows = [ShadowSystemLoadLibrary::class, ShadowMainActivity::class])
class MainActivitySnapshotTest {

    @Test
    fun snapshotIsRestoredOnRecreate() {
        System.setProperty("kistaverk.skipNativeLoad", "true")
        ShadowMainActivity.reset()
        val controller = Robolectric.buildActivity(MainActivity::class.java).setup()
        val activity = controller.get()

        // Force a snapshot from the current state (should be home)
        val snapshot = activity.javaClass
            .getDeclaredMethod("requestSnapshot")
            .apply { isAccessible = true }
            .invoke(activity) as? String
        require(snapshot != null && snapshot.isNotEmpty()) { "Snapshot should not be null" }

        val bundle = Bundle()
        bundle.putString("rust_snapshot", snapshot)

        controller.pause().stop().destroy()

        val restoredController = Robolectric.buildActivity(MainActivity::class.java)
            .create(bundle)
            .start()
            .resume()
            .visible()
        val restored = restoredController.get()

        val root = restored.findViewById<android.widget.FrameLayout>(android.R.id.content)
        assertTrue("Root must have child after restore", root.childCount > 0)

        val json = (restored.javaClass.getDeclaredMethod("requestSnapshot")
            .apply { isAccessible = true }
            .invoke(restored) as? String).orEmpty()
        assertTrue("Snapshot after restore must be valid JSON", runCatching { JSONObject(json) }.isSuccess)
        assertTrue("Snapshot should have been requested", ShadowMainActivity.actions.contains("snapshot"))
        assertTrue("Restore should have been invoked", ShadowMainActivity.actions.contains("restore_state"))

        restoredController.pause().stop().destroy()
    }
}
