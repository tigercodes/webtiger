// A link's menu and a link asked for in a new tab (018), with no WebView behind the adapter:
// who is told of a long press on a link, and what the core is told when the person chooses.
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

class LinksTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val engine = SystemWebViewEngine(pages).also { it.attach(core) }

    /** The long presses the engine told of, in order: the web view, and where the link leads. */
    private val pressed = ArrayList<Pair<Long, String>>()

    private suspend fun aWebView(id: Long): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    @Test
    fun aLongPressOnALinkIsToldToWhoeverShowsItsMenuAndNotToTheCore() =
        runTest {
            val page = aWebView(4)
            engine.onLinkPressed = { webView, url -> pressed.add(webView to url) }

            page.host.linkPressed(4, LINK)

            assertEquals(listOf(4L to LINK), pressed)
            // A link the WebView could say nothing of leads nowhere.
            page.host.linkPressed(4, "")
            assertEquals(1, pressed.size)
            // A menu is the shell's own: the core hears only of what is chosen from it.
            assertTrue(core.reports.none { it.link_open_requested != null })
        }

    @Test
    fun withNobodyToTellNothingComesOfThePress() =
        runTest {
            val page = aWebView(4)

            page.host.linkPressed(4, LINK)

            assertTrue(core.reports.none { it.link_open_requested != null })
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
}
