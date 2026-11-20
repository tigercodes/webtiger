// A link's menu and a link asked for in a new tab (018), and a picture's menu and a file a
// menu saves (022), with no WebView behind the adapter: who is told of a long press, and what
// the core is told when the person chooses.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.WebViewCreate
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val LINK = "https://example.com/next?page=2"
private const val PICTURE = "https://example.com/pictures/tiger.png"

class LinksTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val engine = SystemWebViewEngine(pages).also { it.attach(core) }

    /** The long presses the engine told of, in order: the web view, and what was pressed. */
    private val pressed = ArrayList<Pair<Long, Pressed>>()

    private suspend fun aWebView(id: Long): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    @Test
    fun aLongPressOnALinkIsToldToWhoeverShowsItsMenuAndNotToTheCore() =
        runTest {
            val page = aWebView(4)
            engine.onPressed = { webView, what -> pressed.add(webView to what) }

            page.host.pressed(4, Pressed(link = LINK))

            assertEquals(listOf(4L to Pressed(link = LINK)), pressed)
            // A link the WebView could say nothing of leads nowhere.
            page.host.pressed(4, Pressed())
            assertEquals(1, pressed.size)
            // A menu is the shell's own: the core hears only of what is chosen from it.
            assertTrue(core.reports.none { it.link_open_requested != null })
        }

    @Test
    fun aLongPressOnAPictureIsToldWithThePicturesAddressAndTheLinksWhenItIsOne() =
        runTest {
            val page = aWebView(4)
            engine.onPressed = { webView, what -> pressed.add(webView to what) }

            page.host.pressed(4, Pressed(picture = PICTURE))
            page.host.pressed(4, Pressed(link = LINK, picture = PICTURE))

            assertEquals(listOf(4L to Pressed(picture = PICTURE), 4L to Pressed(link = LINK, picture = PICTURE)), pressed)

            // A picture a page made itself can be neither opened in a tab nor fetched: it is
            // not told of. One that is a link is told as the link it is.
            pressed.clear()
            page.host.pressed(4, Pressed(picture = "data:image/png;base64,AAAA"))
            page.host.pressed(4, Pressed(picture = "blob:https://example.com/1f0e"))
            page.host.pressed(4, Pressed(link = LINK, picture = "data:image/png;base64,AAAA"))
            assertEquals(listOf(4L to Pressed(link = LINK)), pressed)
            // And nothing of any of it is the core's.
            assertTrue(core.reports.isEmpty())
        }

    @Test
    fun withNobodyToTellNothingComesOfThePress() =
        runTest {
            val page = aWebView(4)

            page.host.pressed(4, Pressed(link = LINK, picture = PICTURE))

            assertTrue(core.reports.isEmpty())
        }

    @Test
    fun aLinkAskedForInANewTabIsReportedAsThePersonsDoing() =
        runTest {
            aWebView(4)

            engine.openLink(4, LINK, background = true)
            engine.openLink(4, LINK, background = false)

            val asked = core.reports.filter { it.link_open_requested != null }
            assertEquals("of the web view whose page has the link", listOf(4L, 4L), asked.map { it.web_view_id })
            assertEquals(listOf(LINK, LINK), asked.map { it.link_open_requested?.url })
            assertEquals("behind, or in front, as was chosen", listOf(true, false), asked.map { it.link_open_requested?.background })
            assertTrue("chosen from a menu: the person did it", asked.all { it.link_open_requested?.user_gesture == true })
        }

    @Test
    fun aFileAMenuSavesIsOfferedToTheCoreAsThePersonsOwn() =
        runTest {
            val page = aWebView(4)

            engine.saveFile(4, PICTURE)

            assertEquals("the page of that web view is asked to offer it", listOf(PICTURE), page.saved)
            val asked = core.reports.single { it.download_requested != null }
            assertEquals("of the web view whose page has the picture", 4L, asked.web_view_id)
            assertEquals(PICTURE, asked.download_requested?.url)
            assertEquals("the person chose it: no page asked", true, asked.download_requested?.user_asked)
            assertEquals("no site has suggested a name yet", "", asked.download_requested?.suggested_filename)
        }

    @Test
    fun onlyAnAddressOfTheWebIsSavedAndOnlyForAWebViewThereIs() =
        runTest {
            val page = aWebView(4)

            for (address in listOf("data:text/plain,made%20by%20the%20page", "blob:https://example.com/1f0e", "file:///etc/hosts", "javascript:void(0)", "")) {
                engine.saveFile(4, address)
            }
            engine.saveFile(9, PICTURE)

            assertEquals(emptyList<String>(), page.saved)
            assertTrue(core.reports.isEmpty())
        }

    @Test
    fun aSearchForSelectedWordsIsReportedAsThePersonsDoing() =
        runTest {
            val page = aWebView(4)

            page.host.searchFor(4, "tigers in the wild")

            val asked = core.reports.single { it.search_requested != null }
            assertEquals("of the web view whose page has the words", 4L, asked.web_view_id)
            assertEquals("the words as they were selected: the core tidies them", "tigers in the wild", asked.search_requested?.text)
            assertEquals("chosen from the bar for selected text: the person did it", true, asked.search_requested?.user_gesture)
        }
}
