// Snapshot testing without a library (004 §4): draw a view into a picture on the JVM, and
// compare it with the one checked in. Robolectric's native graphics draw what Android draws.
package com.tigercodes.webtiger.ui

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.view.View
import java.io.File
import kotlin.math.abs
import org.junit.Assert.fail

internal object Snapshots {
    /** A pixel differs when a channel differs by more than this, of 255 (004 §4.1). */
    const val CHANNEL_TOLERANCE = 24

    /** Two pictures match when fewer than this share of their pixels differ. */
    const val SHARE_TOLERANCE = 0.01

    /** The folder for these pictures: they are drawn by Robolectric, as this Android. */
    private val system = "robolectric-${android.os.Build.VERSION.SDK_INT}"

    val isRecording: Boolean get() = System.getenv("WEBTIGER_RECORD_SNAPSHOTS") == "1"

    private val folder: File get() = File(checkNotNull(System.getProperty("webtiger.snapshots")), system)

    private val failed: File get() = File(checkNotNull(System.getProperty("webtiger.snapshots.failed")), system)

    /** What a view shows, pixel for pixel. */
    fun picture(view: View): Bitmap {
        val bitmap = Bitmap.createBitmap(view.width, view.height, Bitmap.Config.ARGB_8888)
        view.draw(Canvas(bitmap))
        return bitmap
    }

    /** The share of pixels, from 0 to 1, that differ; and a picture of where. 1 when the sizes differ. */
    fun difference(drawn: Bitmap, known: Bitmap): Pair<Double, Bitmap> {
        if (drawn.width != known.width || drawn.height != known.height) return 1.0 to drawn
        val count = drawn.width * drawn.height
        val ours = IntArray(count).also { drawn.getPixels(it, 0, drawn.width, 0, 0, drawn.width, drawn.height) }
        val theirs = IntArray(count).also { known.getPixels(it, 0, known.width, 0, 0, known.width, known.height) }
        var differing = 0
        val marked = IntArray(count)
        for (index in 0 until count) {
            val differs = (0..24 step 8).any { shift -> abs(((ours[index] shr shift) and 0xFF) - ((theirs[index] shr shift) and 0xFF)) > CHANNEL_TOLERANCE }
            if (differs) differing += 1
            // What is the same is faded, so what differs stands out.
            marked[index] = if (differs) 0xFFFF00FF.toInt() else (0xFF000000.toInt() or ((ours[index] shr 1) and 0x7F7F7F) + 0x808080)
        }
        return differing.toDouble() / count to Bitmap.createBitmap(marked, drawn.width, drawn.height, Bitmap.Config.ARGB_8888)
    }

    /**
     * Compares a picture with the one checked in as `name`, or writes it when recording. A
     * picture that is not there yet is written, and the test fails: a new one is looked at
     * before it is trusted.
     */
    fun verify(drawn: Bitmap, name: String) {
        val reference = File(folder, "$name.png")
        if (isRecording || !reference.exists()) {
            write(drawn, reference)
            if (!isRecording) fail("$name had no picture; one was written to ${reference.path}: look at it, then commit it")
            return
        }
        val known = checkNotNull(BitmapFactory.decodeFile(reference.path)) { "$name's picture could not be read" }
        val (share, marked) = difference(drawn, known)
        if (share < SHARE_TOLERANCE) return
        write(drawn, File(failed, "$name.actual.png"))
        write(marked, File(failed, "$name.diff.png"))
        fail("$name is not the picture checked in: ${"%.2f".format(share * 100)} % of its pixels differ (see ${failed.path}/$name.actual.png and .diff.png)")
    }

    private fun write(bitmap: Bitmap, file: File) {
        file.parentFile?.mkdirs()
        file.outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
    }
}
