// A view's picture as the adapter hands it to the core (059), with no WebView: any view is
// drawn the same way. On Robolectric's Android with its real pictures, so that a JPEG is a
// JPEG and a smaller one is smaller.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.view.View
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class PicturesTest {
    /**
     * A view that draws a page of sorts. Plain: nothing of its own but a dark bar across its
     * top quarter and an orange square in its bottom right corner, a tenth of its width a side.
     * Noisy: no two dots alike, which is as large a JPEG as its size can be.
     */
    private class Drawn(context: Context, private val noisy: Boolean) : View(context) {
        override fun onDraw(canvas: Canvas) {
            if (noisy) {
                val pixels = IntArray(width * height)
                var seed = 12345L
                for (index in pixels.indices) {
                    seed = seed * 6364136223846793005L + 1442695040888963407L
                    pixels[index] = (seed ushr 24).toInt() or (0xFF shl 24)
                }
                canvas.drawBitmap(Bitmap.createBitmap(pixels, width, height, Bitmap.Config.ARGB_8888), 0f, 0f, null)
                return
            }
            val paint = Paint()
            paint.color = BAR
            canvas.drawRect(0f, 0f, width.toFloat(), height / 4f, paint)
            paint.color = SQUARE
            canvas.drawRect(width * 0.9f, height - width * 0.1f, width.toFloat(), height.toFloat(), paint)
        }
    }

    private fun view(width: Int, height: Int, noisy: Boolean = false): View =
        Drawn(ApplicationProvider.getApplicationContext(), noisy).also {
            it.measure(View.MeasureSpec.makeMeasureSpec(width, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(height, View.MeasureSpec.EXACTLY))
            it.layout(0, 0, width, height)
        }

    /** The picture read back: it is a JPEG, and of the size it says. */
    private fun decoded(pictured: Pictured?): Bitmap {
        val taken = checkNotNull(pictured) { "a picture was made" }
        assertArrayEquals("it begins as every JPEG does", byteArrayOf(0xFF.toByte(), 0xD8.toByte(), 0xFF.toByte()), taken.jpeg.copyOf(3))
        val bitmap = checkNotNull(BitmapFactory.decodeByteArray(taken.jpeg, 0, taken.jpeg.size)) { "the JPEG could not be read back" }
        assertEquals("as wide as it says", taken.width, bitmap.width)
        assertEquals("as tall as it says", taken.height, bitmap.height)
        return bitmap
    }

    /** Whether a dot of a picture is that colour, as nearly as a JPEG keeps one. */
    private fun isNearly(colour: Int, picture: Bitmap, x: Int, y: Int): Boolean {
        val dot = picture.getPixel(x, y)
        return listOf(Color.red(dot) - Color.red(colour), Color.green(dot) - Color.green(colour), Color.blue(dot) - Color.blue(colour)).all { it in -24..24 }
    }

    /** The plain page is in the picture: its bar, its square, and white between, each where it was drawn. */
    private fun assertThePlainPage(picture: Bitmap) {
        val (wide, tall) = picture.width to picture.height
        assertTrue("the bar at the top", isNearly(BAR, picture, wide / 2, tall / 8))
        assertTrue("white under it: a view with no colour of its own is on white", isNearly(Color.WHITE, picture, wide / 2, tall / 2))
        assertTrue("the square in the corner", isNearly(SQUARE, picture, wide - wide / 20, tall - wide / 20))
        assertTrue("and white beside it", isNearly(Color.WHITE, picture, wide - wide / 5, tall - wide / 20))
    }

    @Test
    fun aViewNoWiderThanAskedIsPicturedAtItsOwnSize() {
        val taken = Pictures.of(view(400, 300), mostWide = 800, mostBytes = MOST_BYTES)
        val picture = decoded(taken)
        assertEquals(400 to 300, picture.width to picture.height)
        assertThePlainPage(picture)
        assertTrue("of no more bytes than asked", checkNotNull(taken).jpeg.size <= MOST_BYTES)
        // As wide as asked exactly: as it is.
        assertEquals(800 to 1777, decoded(Pictures.of(view(800, 1777), 800, MOST_BYTES)).let { it.width to it.height })
    }

    @Test
    fun aWiderViewIsDrawnSmallerTheWholeOfItInItsShape() {
        // A phone's window: 1080 by 2400, three dots to a point.
        val phone = decoded(Pictures.of(view(1080, 2400), mostWide = 800, mostBytes = MOST_BYTES))
        assertEquals(800 to 1778, phone.width to phone.height)
        assertThePlainPage(phone)
        // And one twice as wide as asked.
        val wide = decoded(Pictures.of(view(1600, 1200), mostWide = 800, mostBytes = MOST_BYTES))
        assertEquals(800 to 600, wide.width to wide.height)
        assertThePlainPage(wide)
        // Asked for a narrow one.
        val narrow = decoded(Pictures.of(view(1600, 1200), mostWide = 200, mostBytes = MOST_BYTES))
        assertEquals(200 to 150, narrow.width to narrow.height)
        assertThePlainPage(narrow)
    }

    @Test
    fun aViewWithNoSizeHasNoPictureNorHasOneAskedForWithNoWidthOrNoBytes() {
        val unsized = Drawn(ApplicationProvider.getApplicationContext(), noisy = false)
        assertNull("never given a size: it has drawn nothing", Pictures.of(unsized, 800, MOST_BYTES))
        assertNull(Pictures.of(view(400, 0), 800, MOST_BYTES))
        assertNull(Pictures.of(view(0, 300), 800, MOST_BYTES))
        assertNull(Pictures.of(view(400, 300), 0, MOST_BYTES))
        assertNull(Pictures.of(view(400, 300), -1, MOST_BYTES))
        assertNull(Pictures.of(view(400, 300), 800, 0))
        assertNull(Pictures.of(view(400, 300), 800, -1))
        // One dot is a picture.
        assertEquals(1 to 1, decoded(Pictures.of(view(1, 1), 800, MOST_BYTES)).let { it.width to it.height })
    }

    @Test
    fun aPictureOfTooManyBytesIsMadeSmallerAStepAtATimeAndIsNoneWhenNoStepFits() {
        val noise = view(800, 600, noisy = true)
        // With all the bytes it could want: the first step, the whole width at the best a picture is made.
        val first = checkNotNull(Pictures.of(noise, 800, Int.MAX_VALUE))
        assertEquals(800 to 600, decoded(first).let { it.width to it.height })
        assertTrue("noise is a large JPEG: ${first.jpeg.size} bytes", first.jpeg.size > MOST_BYTES)
        // Asked for exactly the bytes it comes to: the same.
        assertArrayEquals(first.jpeg, checkNotNull(Pictures.of(noise, 800, first.jpeg.size)).jpeg)
        // A byte fewer than each step comes to is the next step: as wide and a poorer JPEG;
        // three quarters as wide; half as wide; half as wide and poorer still. Each is under
        // what was asked, and a picture.
        var most = first.jpeg.size - 1
        val widths = ArrayList<Int>()
        repeat(Pictures.STEPS.size - 1) {
            val next = checkNotNull(Pictures.of(noise, 800, most)) { "a step fits under $most bytes" }
            val picture = decoded(next)
            assertTrue("${next.jpeg.size} bytes, asked for no more than $most", next.jpeg.size <= most)
            assertEquals("in the view's shape", 0.75, picture.height.toDouble() / picture.width, 0.01)
            widths.add(next.width)
            most = next.jpeg.size - 1
        }
        assertEquals(listOf(800, 600, 400, 400), widths)
        // And a byte fewer than the last step: none.
        assertNull("no step is as small as $most bytes", Pictures.of(noise, 800, most))
        // The steps are the ones the WebKit adapter goes by.
        assertEquals(listOf(1.0f to 60, 1.0f to 40, 0.75f to 40, 0.5f to 40, 0.5f to 25), Pictures.STEPS)
    }

    private companion object {
        const val MOST_BYTES = 96 * 1024
        val BAR = Color.rgb(0x20, 0x30, 0x40)
        val SQUARE = Color.rgb(0xE0, 0x6A, 0x10)
    }
}
