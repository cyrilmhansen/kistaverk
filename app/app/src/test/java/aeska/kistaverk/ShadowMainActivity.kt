package aeska.kistaverk

import org.json.JSONObject
import org.robolectric.annotation.Implementation
import org.robolectric.annotation.Implements

import org.robolectric.shadows.ShadowActivity

@Implements(MainActivity::class)
class ShadowMainActivity : ShadowActivity() {
    companion object {
        val actions: MutableList<String> = mutableListOf()
        fun reset() = actions.clear()
    }

    @Implementation
    fun dispatch(input: String): String {
        val action = runCatching { JSONObject(input).optString("action") }.getOrDefault("")
        actions.add(action)
        return when (action) {
            "snapshot" -> """{ "snapshot": "{\"type\":\"Column\",\"children\":[]}" }"""
            "restore_state" -> """{ "type": "Column", "children": [] }"""
            else -> """{ "type": "Column", "children": [] }"""
        }
    }
}
