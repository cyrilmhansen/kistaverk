package aeska.kistaverk

import org.robolectric.annotation.Implementation
import org.robolectric.annotation.Implements

@Implements(value = java.lang.System::class, isInAndroidSdk = false)
object ShadowSystemLoadLibrary {
    @Implementation
    @JvmStatic
    fun loadLibrary(libName: String) {
        // No-op in tests to avoid requiring native .so
    }
}
