// A page's icon as the adapter hands it over (046), with no WebView: the picture made a small
// PNG. On Robolectric's Android with its real pictures, so that a PNG is a PNG.
package com.tigercodes.webtiger.webview

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Color
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class IconsTest {
    /** A picture of one colour. */
    private fun picture(width: Int, height: Int, colour: Int = Color.rgb(0xE0, 0x6A, 0x10)): Bitmap =
        Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also { it.eraseColor(colour) }

    /** A picture of which no two pixels are alike enough to pack: as large a PNG as its size can be. */
    private fun noise(width: Int, height: Int): Bitmap {
        val pixels = IntArray(width * height)
        var seed = 12345L
        for (index in pixels.indices) {
            seed = seed * 6364136223846793005L + 1442695040888963407L
            pixels[index] = (seed ushr 24).toInt() or (0xFF shl 24)
        }
        return Bitmap.createBitmap(pixels, width, height, Bitmap.Config.ARGB_8888)
    }

    private fun decoded(png: ByteArray?): Bitmap {
        assertNotNull("a PNG was made", png)
        val bytes = checkNotNull(png)
        assertArrayEquals("it begins as every PNG does", byteArrayOf(0x89.toByte(), 'P'.code.toByte(), 'N'.code.toByte(), 'G'.code.toByte()), bytes.copyOf(4))
        return checkNotNull(BitmapFactory.decodeByteArray(bytes, 0, bytes.size)) { "the PNG could not be read back" }
    }

    @Test
    fun aSmallPictureIsHandedOverAsItIs() {
        val orange = Color.rgb(0xE0, 0x6A, 0x10)
        for ((width, height) in listOf(16 to 16, 32 to 32, 128 to 128, 128 to 20, 1 to 1)) {
            val back = decoded(iconPng(picture(width, height, orange)))
            assertEquals("${width}x$height", width to height, back.width to back.height)
            assertEquals("its colour is the picture's", orange, back.getPixel(0, 0))
        }
    }

    @Test
    fun aLargePictureIsMadeSmallerInItsOwnShape() {
        // The longer side is 128; the other keeps its share, and is never nothing.
        for ((given, made) in listOf((512 to 512) to (128 to 128), (512 to 256) to (128 to 64), (192 to 640) to (38 to 128), (129 to 129) to (128 to 128), (4000 to 3) to (128 to 1))) {
            val picture = picture(given.first, given.second)
            val back = decoded(iconPng(picture))
            assertEquals("${given.first}x${given.second}", made, back.width to back.height)
            assertTrue("the picture that was handed in is still the WebView's to use", !picture.isRecycled)
        }
    }

    @Test
    fun whatTheCoreWouldNotKeepIsNotHandedOver() {
        // A picture that has been given back to the system cannot be read.
        assertNull(iconPng(picture(32, 32).also { it.recycle() }))
        // One whose PNG is more bytes than the core keeps: noise, which does not pack. With
        // the most that is kept put at what this PNG is, it is handed over; at one byte
        // fewer, it is not.
        val noisy = noise(64, 64)
        val png = checkNotNull(iconPng(noisy)) { "the largest PNG a picture of 64 pixels can be is one the core keeps" }
        assertTrue("noise does not pack: ${png.size} bytes", png.size > 8000)
        assertTrue(png.size <= ICON_BYTES)
        assertNotNull(iconPng(noisy, mostBytes = png.size))
        assertNull(iconPng(noisy, mostBytes = png.size - 1))
    }
}
