// What the device's WebView can do for a sleeping tab's picture (059 §5.1), tried against the
// suite's own pages before anything is built on it: whether a web view that is in no window,
// as a tab behind is, can be pictured at all; that the picture is of the page and not of
// nothing; how long it takes; and how large it is as a JPEG no wider than 800 pixels. The
// Apple suite's WebKitPictureTests.swift, for Android.
package com.tigercodes.webtiger.webview

import android.graphics.Bitmap
import android.graphics.Canvas
import android.util.Log
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import java.io.ByteArrayOutputStream
import kotlin.math.abs
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewPictureTest : AdapterTestCase() {
    /** The share of a picture's pixels that are not the colour of its first: of a page on white, how much of it is drawn on. */
    private fun inked(picture: Bitmap): Double {
        val pixels = IntArray(picture.width * picture.height)
        picture.getPixels(pixels, 0, picture.width, 0, 0, picture.width, picture.height)
        val first = pixels[0]
        fun far(a: Int, b: Int) = (0..2).any { abs(((a shr (it * 8)) and 0xFF) - ((b shr (it * 8)) and 0xFF)) > 24 }
        return pixels.count { far(it, first) }.toDouble() / pixels.size
    }

    /**
     * A web view drawn into a bitmap, as it is now, no wider than so many pixels: the view is
     * drawn smaller where it is wider, the whole of it. The picture, and how long it took.
     */
    private fun picture(view: WebView, mostWide: Int = MOST_WIDE): Pair<Bitmap, Long> =
        onMain {
            val began = System.nanoTime()
            val wide = minOf(view.width, mostWide)
            val by = wide.toFloat() / view.width
            val bitmap = Bitmap.createBitmap(wide, maxOf(1, (view.height * by).toInt()), Bitmap.Config.ARGB_8888)
            view.draw(Canvas(bitmap).apply { scale(by, by) })
            bitmap to (System.nanoTime() - began) / 1_000_000
        }

    private fun jpeg(picture: Bitmap): Int = ByteArrayOutputStream().also { picture.compress(Bitmap.CompressFormat.JPEG, 60, it) }.size()

    /** A tab that was in front, and is behind: its web view was in a window, and is in none. */
    @Test
    fun aWebViewThatWasShownAndIsInNoWindowCanBePicturedAndThePictureIsOfItsPage() {
        assertNotNull(create(1, newLoginSet()).ack)
        show(1)
        load(1, site.url("/pictures"))
        await(1, "the page to finish") { finished(it) }
        val view = checkNotNull(webView(1))
        // Drawn once on the screen, as a tab in front is.
        Thread.sleep(1_000)
        onMain { (view.parent as ViewGroup).removeView(view) }
        assertNull("in no window, as a tab that is behind is", onMain { view.parent })
        assertTrue(onMain { !view.isAttachedToWindow })

        // As large as the view is, to see what a screen's worth of pixels costs; and as it would be kept.
        val (whole, _) = picture(view, mostWide = Int.MAX_VALUE)
        val (bitmap, took) = picture(view)
        val share = inked(bitmap)
        val bytes = jpeg(bitmap)
        Log.i(
            TAG,
            "shown, then in no window: ${whole.width} by ${whole.height} pixels as it is (${jpeg(whole)} bytes as a JPEG), " +
                "${bitmap.width} by ${bitmap.height} as kept ($bytes bytes), drawn in $took ms, ${(share * 1000).toInt()} thousandths drawn on",
        )
        assertEquals("no wider than is kept", MOST_WIDE, bitmap.width)
        assertEquals("the shape the web view has", whole.height.toDouble() / whole.width, bitmap.height.toDouble() / bitmap.width, 0.01)
        assertTrue("the picture is of the page, and not of nothing: $share drawn on", share > 0.005)
        assertTrue("a page of pictures and words, as a JPEG no wider than 800 pixels: $bytes bytes", bytes < MOST_BYTES / 2)
    }

    /** A tab that was opened behind: its web view was never in a window. Given a size, as its place in the window would give it. */
    @Test
    fun aWebViewThatWasNeverShownCanBePicturedToo() {
        assertNotNull(create(2, newLoginSet()).ack)
        val view = checkNotNull(webView(2))
        onMain {
            view.measure(View.MeasureSpec.makeMeasureSpec(800, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(1200, View.MeasureSpec.EXACTLY))
            view.layout(0, 0, 800, 1200)
        }
        load(2, site.url("/pictures"))
        await(2, "the page to finish") { finished(it) }
        Thread.sleep(1_000)
        assertTrue(onMain { !view.isAttachedToWindow })

        val (bitmap, took) = picture(view)
        val share = inked(bitmap)
        Log.i(
            TAG,
            "never shown: ${bitmap.width} by ${bitmap.height} pixels, ${jpeg(bitmap)} bytes as a JPEG, drawn in $took ms, " +
                "${(share * 1000).toInt()} thousandths drawn on",
        )
        assertTrue(bitmap.width == 800 && bitmap.height == 1200)
        assertTrue("the picture is of the page, and not of nothing: $share drawn on", share > 0.005)
    }

    private companion object {
        const val TAG = "WebTigerPicture"

        /** The most bytes a picture may be, and the most pixels wide (059 D4). */
        const val MOST_BYTES = 96 * 1024
        const val MOST_WIDE = 800
    }
}
