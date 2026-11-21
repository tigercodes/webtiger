// A link pressed in another app (031): what the window reads of what the system hands it, what
// it then asks of the core, and what the system is told the app shows. Nothing is loaded: the
// core here is a script.
package com.tigercodes.webtiger

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.pm.PackageManager
import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.unconfinedSession
import com.tigercodes.webtiger.ui.windowScript
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ViewAddressTest {
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** What the system hands the app that shows an address: the address, as another app gave it. */
    private fun pressed(address: String?, action: String = Intent.ACTION_VIEW): Intent = Intent(action).apply { if (address != null) data = Uri.parse(address) }

    /** The addresses the core was asked to open, with the window they were for, in order. */
    private fun opened(core: ScriptedCore): List<Pair<Long, String>> = core.requests.mapNotNull { it.open_address?.let { asked -> asked.window_id to asked.url } }

    @Test
    fun theAddressIsReadOnlyFromTheSystemsOwnRequestAsItWasGiven() {
        assertEquals("https://example.com/story?id=7#top", MainActivity.address(pressed("https://example.com/story?id=7#top")))
        // As it was given: nothing is mended, and what it is, is the core's to say.
        assertEquals("HTTP://Example.COM", MainActivity.address(pressed("HTTP://Example.COM")))
        assertEquals("javascript:alert(1)", MainActivity.address(pressed("javascript:alert(1)")))

        assertNull("no address", MainActivity.address(pressed(null)))
        assertNull(MainActivity.address(pressed("")))
        assertNull(MainActivity.address(pressed("  ")))
        assertNull(MainActivity.address(null))
        // Anything else that carries an address is not a link being pressed.
        for (other in listOf(Intent.ACTION_SEND, Intent.ACTION_MAIN, Intent.ACTION_WEB_SEARCH, Intent.ACTION_PROCESS_TEXT, MainActivity.ACTION_SEARCH_WORDS)) {
            assertNull(other, MainActivity.address(pressed("https://example.com/", action = other)))
        }
        // An address as long as may be handed on is; a longer one is not handed on at all,
        // and not cut: a part of an address is another address.
        val longest = "https://example.com/" + "a".repeat(MainActivity.LONGEST_ADDRESS - 20)
        assertEquals(longest, MainActivity.address(pressed(longest)))
        assertNull(MainActivity.address(pressed(longest + "a")))
    }

    @Test
    fun theWindowHandsTheAddressToTheCoreWithItsWindowAndNothingElseOfWhatItWasHanded() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()

            // An address of the web, and what is none: each handed on as it is. Which of them
            // is opened is the core's rule.
            val addresses = listOf("https://example.com/story", "example.com", "javascript:alert(1)", "file:///sdcard/page.html")
            for (address in addresses) {
                // With what another app can put beside a link: headers to send, the page the
                // link was on, its own name, and an address of some other kind as "words".
                val handed =
                    pressed(address)
                        .putExtra("com.android.browser.headers", android.os.Bundle().apply { putString("Cookie", "who=somebody") })
                        .putExtra(Intent.EXTRA_REFERRER, Uri.parse("android-app://com.example.other"))
                        .putExtra("com.android.browser.application_id", "com.example.other")
                        .putExtra(MainActivity.EXTRA_WORDS, "tigers in the wild")
                MainActivity.handedOver(handed, model)
            }

            assertEquals(addresses.map { 1L to it }, opened(core))
            // The address and its window are all the core is told: nothing else was asked for
            // them. No tab opened by the shell, nothing typed for the core to mend, nothing
            // searched for.
            val asked = core.requests.filter { it.open_address != null }
            assertEquals(addresses.size, core.requests.size - core.requests.indexOf(asked.first()))
            assertTrue(core.requests.none { it.open_tab != null || it.submit_address != null || it.search_words != null })
        }

    @Test
    fun anIntentThatAsksForNoAddressToBeShownAsksTheCoreNothing() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()

            MainActivity.handedOver(null, model)
            MainActivity.handedOver(Intent(Intent.ACTION_MAIN), model)
            MainActivity.handedOver(pressed(null), model)
            MainActivity.handedOver(pressed("  "), model)
            // The app's own launch, with an address in it, is not a link being pressed.
            MainActivity.handedOver(pressed("https://example.com/", action = Intent.ACTION_MAIN), model)
            MainActivity.handedOver(pressed("https://example.com/", action = Intent.ACTION_SEND), model)
            // Words are words: an address among them is not opened as one.
            MainActivity.handedOver(SearchWordsActivity.windowIntent(context, "https://example.com/"), model)

            assertEquals(emptyList<Pair<Long, String>>(), opened(core))
            assertEquals(listOf("https://example.com/"), core.requests.mapNotNull { it.search_words?.text })
        }

    @Test
    fun whatWasHandedOverIsNotHandedOverAgainWhenTheWindowIsMadeAgainFromRecentApps() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            // The system starts a window again from the list of recent apps with the intent it
            // was first started with, and says so: the link was opened, or the words searched
            // for, the first time.
            val again = Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY

            MainActivity.handedOver(pressed("https://example.com/story").addFlags(again), model)
            MainActivity.handedOver(SearchWordsActivity.windowIntent(context, "tigers in the wild").addFlags(again), model)

            assertEquals(emptyList<Pair<Long, String>>(), opened(core))
            assertTrue(core.requests.none { it.search_words != null })
            // The first time, each is handed over.
            MainActivity.handedOver(pressed("https://example.com/story"), model)
            MainActivity.handedOver(SearchWordsActivity.windowIntent(context, "tigers in the wild"), model)
            assertEquals(listOf(1L to "https://example.com/story"), opened(core))
            assertEquals(listOf("tigers in the wild"), core.requests.mapNotNull { it.search_words?.text })
        }

    @Test
    fun addressesThatComeBeforeTheWindowIsOpenWaitForItEachOfThemInOrder() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))

            // Several links at once, as an app that opens a list of them sends them.
            model.openFromOutside("https://example.com/first")
            model.openFromOutside("https://example.com/second")
            model.openFromOutside("https://example.com/first")
            model.openFromOutside("https://example.com/third")
            assertEquals("there is no window to open them in yet", emptyList<Pair<Long, String>>(), opened(core))

            model.open()

            // Each of them, once, in the order they came, in the window that opened: one that
            // came twice is opened twice.
            val came = listOf("first", "second", "first", "third").map { 1L to "https://example.com/$it" }
            assertEquals(came, opened(core))
            model.openFromOutside("https://example.org/")
            assertEquals(came + (1L to "https://example.org/"), opened(core))

            // A window that is closed and opened again does not open them a second time.
            model.close()
            model.open()
            assertEquals(came + (1L to "https://example.org/"), opened(core))
        }

    @Test
    fun theSystemIsToldWebTigerShowsAnAddressOfTheWebAndOfNoOtherKind() {
        fun answers(address: String, vararg categories: String): List<ActivityInfo> {
            val asked = Intent(Intent.ACTION_VIEW, Uri.parse(address)).apply { categories.forEach { addCategory(it) } }
            return context.packageManager.queryIntentActivities(asked, PackageManager.MATCH_DEFAULT_ONLY).map { it.activityInfo }.filter { it.packageName == context.packageName }
        }

        // An address of the web, of any site, as a link pressed in another app is asked
        // after: with nothing said of it, and as something a page may link to. The window
        // answers, and nothing else of the app's.
        for (address in listOf("https://example.com/", "http://example.com/story?id=7", "https://another.example.org:8443/")) {
            assertEquals(address, listOf(MainActivity::class.java.name), answers(address).map { it.name })
            assertEquals(address, listOf(MainActivity::class.java.name), answers(address, Intent.CATEGORY_BROWSABLE).map { it.name })
        }
        // Nothing else: not a file, not what an app keeps, not a letter or a number, not a
        // script, not an app's own scheme.
        for (other in listOf("file:///sdcard/page.html", "content://media/external/file/1", "mailto:somebody@example.com", "tel:+15550100", "javascript:alert(1)", "data:text/html,hello", "about:blank", "ftp://example.com/file", "webtiger://example.com/")) {
            assertEquals(other, emptyList<String>(), answers(other).map { it.name })
        }

        val window = context.packageManager.getActivityInfo(ComponentName(context, MainActivity::class.java), 0)
        assertTrue("other apps can start it: that is what a browser is for", window.exported)
        // The system's list of browsers names it as the app.
        assertEquals("WebTiger", window.loadLabel(context.packageManager).toString())
        // There is one window, however fast and however often it is asked for: every request
        // reaches it, and none starts a second.
        assertEquals(ActivityInfo.LAUNCH_SINGLE_TASK, window.launchMode)
    }
}
