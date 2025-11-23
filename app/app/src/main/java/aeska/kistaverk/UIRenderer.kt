package aeska.kistaverk

import android.content.Context
import android.graphics.Color
import android.view.View
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import org.json.JSONObject

// Ajout du callback 'onAction' : (String) -> Unit
class UiRenderer(
    private val context: Context,
    private val onAction: (String) -> Unit
) {

    fun render(jsonString: String): View {
        return createView(JSONObject(jsonString))
    }

    private fun createView(data: JSONObject): View {
        val type = data.optString("type")
        return when (type) {
            "Column" -> createColumn(data)
            "Text" -> createText(data)
            "Button" -> createButton(data)
            else -> createErrorView("Unknown: $type")
        }
    }

    // ... (createColumn et createText ne changent pas, sauf createView récursif) ...
    // ATTENTION : Pour createColumn, assure-toi de bien rappeler createView
    // Je remets le code abrégé pour être clair :

    private fun createColumn(data: JSONObject): View {
        val layout = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        if (data.has("padding")) {
            val p = data.getInt("padding")
            layout.setPadding(p, p, p, p)
        }
        val children = data.optJSONArray("children")
        if (children != null) {
            for (i in 0 until children.length()) {
                layout.addView(createView(children.getJSONObject(i)))
            }
        }
        return layout
    }

    private fun createText(data: JSONObject): View {
        return TextView(context).apply {
            text = data.optString("text")
            textSize = data.optDouble("size", 14.0).toFloat()
        }
    }

    // --- C'EST ICI QUE CA CHANGE ---
    private fun createButton(data: JSONObject): View {
        val btn = Button(context)
        btn.text = data.optString("text")

        // On récupère l'action définie dans le JSON Rust (ex: "increment")
        val actionName = data.optString("action")

        btn.setOnClickListener {
            // Au lieu du Toast, on remonte l'info via le callback
            onAction(actionName)
        }
        return btn
    }

    private fun createErrorView(msg: String): View {
        return TextView(context).apply {
            text = msg
            setTextColor(Color.RED)
        }
    }
}