// Sites' icons (046), the window's part: the pictures of the numbers the core gives tabs'
// icons, asked for once and kept while the window is open. Which tab has which icon, and
// whether a picture is an icon at all, are the core's.
package com.tigercodes.webtiger.ui

import android.graphics.BitmapFactory
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap

/** The most pictures asked for in one question: what an answer carries. */
internal const val ICONS_ASKED = 64

/** The most pictures a window keeps beyond those its tabs show now. */
internal const val ICONS_KEPT = 512

/** The widest and the tallest a picture the core keeps is, in pixels. */
private const val ICON_PIXELS = 256

/**
 * The picture the core's bytes are, to be drawn. Null for bytes that are no picture, and for
 * one larger than the core keeps: its size is read before any of it is decoded.
 */
internal fun iconPicture(png: ByteArray): ImageBitmap? {
    val size = BitmapFactory.Options().apply { inJustDecodeBounds = true }
    BitmapFactory.decodeByteArray(png, 0, png.size, size)
    if (size.outWidth !in 1..ICON_PIXELS || size.outHeight !in 1..ICON_PIXELS) return null
    return BitmapFactory.decodeByteArray(png, 0, png.size)?.asImageBitmap()
}
