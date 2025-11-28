package aeska.kistaverk

import android.view.View
import android.widget.FrameLayout
import android.widget.ScrollView

object TestViews {
    fun unwrap(view: View): View {
        var current: View = view
        while (true) {
            when (current) {
                is ScrollView -> return current
                is FrameLayout -> {
                    val child = current.getChildAt(0) ?: return current
                    if (child === current) return current
                    current = child
                }
                else -> return current
            }
        }
    }
}
