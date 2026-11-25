// A sleeping tab's picture (059), the window's part: the picture the core says stands where
// the page of the tab in front will be, asked for by its number and drawn there until the
// core says none. When there is one, and for how long, are the core's: nothing here decides.
package com.tigercodes.webtiger.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.tigercodes.webtiger.R

/** The widest a picture the core keeps is, in pixels; and the tallest this window draws. */
internal const val PICTURE_MOST_WIDE = 800
internal const val PICTURE_MOST_TALL = 8192

/**
 * The picture the core's bytes are, to be drawn. Null for bytes that are no JPEG, and for a
 * picture wider than the core keeps or taller than any window: its kind and its size are read
 * before any of it is decoded. Null too when the system will not decode it, or has no memory
 * to: a picture that cannot be drawn is no picture, and the window goes on without.
 */
internal fun pictureOfAPage(jpeg: ByteArray): ImageBitmap? =
    try {
        val said = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(jpeg, 0, jpeg.size, said)
        val fits = said.outMimeType == "image/jpeg" && said.outWidth in 1..PICTURE_MOST_WIDE && said.outHeight in 1..PICTURE_MOST_TALL
        if (fits) BitmapFactory.decodeByteArray(jpeg, 0, jpeg.size)?.asImageBitmap() else null
    } catch (_: RuntimeException) {
        null
    } catch (_: OutOfMemoryError) {
        null
    }

/**
 * A page as it was, drawn where the page will be (059): as wide as its place, from the top, in
 * its own shape, and cut off where its place ends. A place that is taller than the picture
 * shows what is under it below. It is a picture and nothing else: a touch goes through it, and
 * a screen reader is told what it is.
 */
@Composable
internal fun PageAsItWas(picture: ImageBitmap, modifier: Modifier = Modifier) {
    Image(
        bitmap = picture,
        contentDescription = stringResource(R.string.page_picture),
        contentScale = ContentScale.FillWidth,
        alignment = Alignment.TopCenter,
        modifier = modifier.fillMaxSize().testTag("page-picture"),
    )
}
