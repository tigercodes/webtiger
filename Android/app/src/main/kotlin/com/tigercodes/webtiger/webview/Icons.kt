// A page's icon, as the System WebView hands it over (046). The WebView fetches a page's icon
// itself, as part of loading the page and in the web view's own login set; the adapter makes
// the picture a small PNG and passes it on. Whether it is kept, and whose it is, is the core's.
package com.tigercodes.webtiger.webview

import android.graphics.Bitmap
import java.io.ByteArrayOutputStream

/** The widest and the tallest an icon is handed over as, in pixels. */
internal const val ICON_PIXELS = 128

/** The most bytes of a picture the core keeps. */
internal const val ICON_BYTES = 65_536

/**
 * The picture as a PNG for the core: as it is when it is at most [ICON_PIXELS] each way; made
 * smaller, in its own shape, when it is larger. Null for a picture of no size, for one that
 * cannot be read, and for one whose PNG is more bytes than the core keeps (`mostBytes`).
 */
internal fun iconPng(icon: Bitmap, mostBytes: Int = ICON_BYTES): ByteArray? {
    if (icon.isRecycled || icon.width <= 0 || icon.height <= 0) return null
    val largest = maxOf(icon.width, icon.height)
    return try {
        val small =
            if (largest <= ICON_PIXELS) {
                icon
            } else {
                Bitmap.createScaledBitmap(icon, maxOf(1, icon.width * ICON_PIXELS / largest), maxOf(1, icon.height * ICON_PIXELS / largest), true)
            }
        val out = ByteArrayOutputStream()
        val written = small.compress(Bitmap.CompressFormat.PNG, 100, out)
        // (The WebView's own picture is the WebView's to keep.)
        if (small !== icon) small.recycle()
        out.toByteArray().takeIf { written && it.size <= mostBytes }
    } catch (_: RuntimeException) {
        null
    }
}
