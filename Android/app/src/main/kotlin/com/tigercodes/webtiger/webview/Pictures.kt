// A web view's picture for the core (059): the view drawn as it is, whether or not it is on
// the screen, no wider than the core asks and a JPEG of no more bytes than it asks. The core
// asks as a tab is put to sleep, and the picture stands where the page will be while the tab
// wakes. This file decides nothing about tabs: it draws.
package com.tigercodes.webtiger.webview

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.view.View
import java.io.ByteArrayOutputStream
import kotlin.math.roundToInt

/** A picture as the core is answered: a JPEG, and its size in pixels. */
internal class Pictured(val jpeg: ByteArray, val width: Int, val height: Int)

internal object Pictures {
    /**
     * How a picture is made smaller until it is of no more bytes than were asked for: how much
     * of its width it keeps, and how good a JPEG it is, in hundredths. A plain page fits at the
     * first; a page of photographs may need the later ones; one that fits at none has no
     * picture. (The WebKit adapter goes by the same steps.)
     */
    val STEPS = listOf(1.0f to 60, 1.0f to 40, 0.75f to 40, 0.5f to 40, 0.5f to 25)

    /**
     * A view's picture as it is now: the whole of it, drawn smaller where it is wider than
     * `mostWide` pixels, in its shape. Null when there is none to give (the view was never
     * given a size and has drawn nothing), or none of so few bytes.
     */
    fun of(view: View, mostWide: Int, mostBytes: Int): Pictured? {
        if (mostWide <= 0 || view.width <= 0 || view.height <= 0) return null
        for ((part, quality) in STEPS) {
            val wide = (minOf(view.width, mostWide) * part).roundToInt().coerceAtLeast(1)
            val by = wide.toFloat() / view.width
            val tall = (view.height * by).roundToInt().coerceAtLeast(1)
            val bitmap = Bitmap.createBitmap(wide, tall, Bitmap.Config.ARGB_8888)
            val jpeg = ByteArrayOutputStream()
            try {
                // On white, as a page with no colour of its own is: a JPEG has no way to say "nothing here".
                val canvas = Canvas(bitmap)
                canvas.drawColor(Color.WHITE)
                canvas.scale(by, by)
                view.draw(canvas)
                if (!bitmap.compress(Bitmap.CompressFormat.JPEG, quality, jpeg)) return null
            } finally {
                bitmap.recycle()
            }
            if (jpeg.size() <= mostBytes) return Pictured(jpeg.toByteArray(), wide, tall)
        }
        return null
    }
}
