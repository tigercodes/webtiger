// The adapter suite's System WebView run, for a page's icon (046): the WebView fetches the
// icon of the page it shows itself, and the adapter hands it to the core as a small PNG with
// the page it is the icon of. A page that names none hands none over.
package com.tigercodes.webtiger.webview

import android.graphics.BitmapFactory
import android.graphics.Color
import com.tigercodes.webtiger.core.v1.IconReceived
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewIconTest : AdapterTestCase() {
    private fun icons(id: Long = 1): List<IconReceived> = reports(id).mapNotNull { it.icon_received }

    /** A web view on the screen, as a tab's is, with a page of the test site loaded in it. */
    private fun page(path: String, title: String, id: Long = 1) {
        create(id)
        show(id)
        load(id, site.url(path))
        await(id, "the page to load") { finished(it) && hasTitle(title)(it) }
    }

    @Test
    fun aPagesIconIsHandedOverAsASmallPngWithThePageItIsTheIconOf() {
        page("/with-icon", "With icon")
        await(1, "the page's icon") { reports -> reports.any { it.icon_received != null } }

        val icon = icons().first()
        assertEquals("it names the page it is the icon of", site.url("/with-icon"), icon.page_url)
        val bytes = icon.png.toByteArray()
        assertTrue("at most what the core keeps: ${bytes.size} bytes", bytes.size in 1..ICON_BYTES)
        val picture = checkNotNull(BitmapFactory.decodeByteArray(bytes, 0, bytes.size)) { "what was handed over is no picture" }
        assertTrue("of a size the core keeps: ${picture.width}x${picture.height}", picture.width in 1..ICON_PIXELS && picture.height in 1..ICON_PIXELS)
        // The page's own picture: an orange square.
        val middle = picture.getPixel(picture.width / 2, picture.height / 2)
        assertTrue(
            "the page's orange, more or less: ${Integer.toHexString(middle)}",
            Color.red(middle) > 0xC0 && Color.green(middle) in 0x40..0x90 && Color.blue(middle) < 0x40,
        )
    }

    @Test
    fun aPageThatNamesNoIconHandsNoneOver() {
        page("/landed", "Landed")
        // The WebView may ask the site for the icon every site may have, and is told there is
        // none. Whether it asks or not, nothing is handed over.
        repeat(150) {
            if (site.requests.any { it.startsWith("/favicon.ico") }) return@repeat
            Thread.sleep(10)
        }
        Thread.sleep(500)
        assertEquals("no icon was handed over", emptyList<IconReceived>(), icons())
    }
}
