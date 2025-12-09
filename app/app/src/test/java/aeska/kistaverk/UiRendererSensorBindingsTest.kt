package aeska.kistaverk

import android.widget.Button
import android.widget.CheckBox
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UiRendererSensorBindingsTest {

    @Test
    fun startButton_carries_sensor_bindings_and_interval() {
        val actions = mutableListOf<Triple<String, Boolean, Map<String, String>>>()
        val renderer = UiRenderer(ApplicationProvider.getApplicationContext()) { action, picker, _, bindings ->
            actions.add(Triple(action, picker, bindings))
        }

        val ui = """
            {
              "type": "Column",
              "children": [
                { "type": "Checkbox", "bind_key": "sensor_accel", "checked": true },
                { "type": "Checkbox", "bind_key": "sensor_gyro", "checked": true },
                { "type": "Checkbox", "bind_key": "sensor_mag", "checked": true },
                { "type": "Checkbox", "bind_key": "sensor_gps", "checked": false },
                { "type": "Checkbox", "bind_key": "sensor_battery", "checked": true },
                { "type": "TextInput", "bind_key": "sensor_interval_ms", "text": "200" },
                { "type": "Button", "text": "Start logging", "action": "sensor_logger_start" }
              ]
            }
        """.trimIndent()

        val root = TestViews.unwrap(renderer.render(ui)) as ScrollView
        val layout = root.getChildAt(0) as LinearLayout
        val mag = layout.getChildAt(2) as CheckBox
        val gps = layout.getChildAt(3) as CheckBox
        val interval = layout.getChildAt(5) as EditText
        val start = layout.getChildAt(6) as Button

        mag.isChecked = false
        gps.isChecked = true
        interval.setText("750")

        start.performClick()

        val (action, needsPicker, bindings) = actions.last()
        assertEquals("sensor_logger_start", action)
        assertEquals(false, needsPicker)
        assertEquals("true", bindings["sensor_accel"])
        assertEquals("false", bindings["sensor_mag"])
        assertEquals("true", bindings["sensor_gps"])
        assertEquals("750", bindings["sensor_interval_ms"])
    }
}
