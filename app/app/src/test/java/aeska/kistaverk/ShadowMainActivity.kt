package aeska.kistaverk

import org.json.JSONObject
import org.robolectric.annotation.Implementation
import org.robolectric.annotation.Implements

import org.robolectric.shadows.ShadowActivity

@Implements(MainActivity::class)
class ShadowMainActivity : ShadowActivity() {
    @Implementation
    fun dispatch(input: String): String {
        val action = runCatching { JSONObject(input).optString("action") }.getOrDefault("")
        return when (action) {
            "snapshot" -> """{ "snapshot": "{\"type\":\"Column\",\"children\":[]}" }"""
            "restore_state" -> """{ "type": "Column", "children": [] }"""
            else -> """{ "type": "Column", "children": [] }"""
        }
    }
}
