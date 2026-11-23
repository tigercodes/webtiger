// The adapter suite's System WebView run, for a page's icon (046): the WebView fetches the
// icon of the page it shows itself, whether the page carries it or names an address of its
// site, and the adapter hands it to the core as a small PNG with the page it is the icon of.
// A page that names none hands none over.
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
    fun anIconAtAnAddressOfTheSiteIsFetchedByTheWebViewAndHandedOver() {
        page("/with-served-icon", "With served icon")
        await(1, "the page's icon") { reports -> reports.any { it.icon_received != null } }

        // The WebView asked the site for the picture itself: the adapter asked for nothing.
        assertTrue("the site was asked for the icon: ${site.requests}", site.requests.contains("/icon.png"))
        val icon = icons().first()
        assertEquals(site.url("/with-served-icon"), icon.page_url)
        val bytes = icon.png.toByteArray()
        val picture = checkNotNull(BitmapFactory.decodeByteArray(bytes, 0, bytes.size)) { "what was handed over is no picture" }
        assertTrue("of a size the core keeps: ${picture.width}x${picture.height}", picture.width in 1..ICON_PIXELS && picture.height in 1..ICON_PIXELS)
        // The site's own picture: a blue square.
        val middle = picture.getPixel(picture.width / 2, picture.height / 2)
        assertTrue(
            "the site's blue, more or less: ${Integer.toHexString(middle)}",
            Color.blue(middle) > 0xC0 && Color.green(middle) in 0x30..0x80 && Color.red(middle) < 0x40,
        )
    }

    /** The addresses the site was asked for that say with whose cookies an icon was asked for, in order. */
    private fun iconsAskedAs(): List<String> = site.requests.filter { it.startsWith("/icon-of-") }

    private fun awaitIconsAskedAs(count: Int) {
        repeat(500) {
            if (iconsAskedAs().size >= count) return
            Thread.sleep(10)
        }
        throw AssertionError("the site was asked for ${iconsAskedAs().size} icons by cookie, not $count: ${site.requests}")
    }

    /**
     * The WebView fetches a page's icon as the web view's own login set: with its cookies, and
     * with no other's. (An isolation property: an icon is asked for by the engine, not by the
     * page, and must be asked for as the page is.)
     */
    @Test
    fun aPagesIconIsAskedForWithItsOwnLoginSetsCookiesAndNoOthers() {
        val (ada, nobody) = newLoginSet() to newLoginSet()
        // Signed in, in one login set: the site is asked for the icon as ada.
        create(1, loginSet = ada)
        show(1)
        load(1, site.url("/set?v=ada"))
        await(1, "the sign-in") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/with-icon-by-cookie"))
        await(1, "the page") { hasTitle("With icon by cookie")(it) }
        awaitIconsAskedAs(1)
        assertEquals(listOf("/icon-of-ada"), iconsAskedAs())

        // Another login set, at the same page of the same site, while the first is open: the
        // site is asked as nobody.
        create(2, loginSet = nobody)
        show(2)
        load(2, site.url("/with-icon-by-cookie"))
        await(2, "the page") { finished(it) && hasTitle("With icon by cookie")(it) }
        awaitIconsAskedAs(2)
        assertEquals(listOf("/icon-of-ada", "/icon-of-none"), iconsAskedAs())
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
