// A link's menu (018), drawn against a scripted core: when it is shown, what it says, and what
// each choice does.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.click
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LinkOpenRequested
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePage
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

private const val LINK = "https://example.org/articles/a-long-read?from=the-front-page"

@RunWith(RobolectricTestRunner::class)
class LinkMenuTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val core = ScriptedCore(::windowScript)
    private val pages = FakePages()
    private val engine = SystemWebViewEngine(pages)
    private val clipboard = MemoryClipboard()

    @After
    fun end() {
        scope.cancel()
    }

    /** A window whose tab in front shows example.com in the web view 4, and that web view's page as the adapter has it. */
    private fun aWindow(): FakePage {
        val session = BrowserSession(core, engine, scope, clipboard = clipboard)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
        runBlocking {
            for (id in listOf(4L, 5L)) {
                engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"))))
            }
        }
        theCoreSays(snapshot(url = "https://example.com/", display = "example.com", webView = 4))
        return pages.made.first()
    }

    private fun theCoreSays(snapshot: WindowSnapshot) {
        core.emit(snapshot, subscription = 3)
        compose.waitForIdle()
    }

    /** The person presses and holds a link of a web view's page, as the adapter says it. */
    private fun press(page: FakePage, url: String = LINK) {
        page.host.linkPressed(page.id, url)
        compose.waitForIdle()
    }

    private fun menus(): Int = compose.onAllNodesWithTag("link-menu").fetchSemanticsNodes().size

    /** What the core was told was asked for in a new tab. */
    private fun asked(): List<Pair<Long, LinkOpenRequested>> = core.reports.mapNotNull { report -> report.link_open_requested?.let { report.web_view_id to it } }

    @Test
    fun aLinkPressedAndHeldShowsWhereItLeadsAndWhatCanBeDoneWithIt() {
        val page = aWindow()
        assertEquals(0, menus())

        press(page)

        assertEquals(1, menus())
        compose.onNodeWithTag("link-address").assertTextEquals(LINK)
        compose.onNodeWithTag("link-open").assertTextEquals("Open in New Tab")
        compose.onNodeWithTag("link-copy").assertTextEquals("Copy Link")
        compose.onNodeWithTag("link-cancel").assertTextEquals("Cancel")
        // Nothing is asked of the core, and nothing copied, until something is chosen.
        assertEquals(emptyList<Pair<Long, LinkOpenRequested>>(), asked())
        assertNull(clipboard.copied)
    }

    @Test
    fun openInNewTabAsksTheCoreForATabBehindAndPutsTheMenuAway() {
        val page = aWindow()
        press(page)

        compose.onNodeWithTag("link-open").performClick()
        compose.waitForIdle()

        val (webView, link) = asked().single()
        assertEquals("of the web view whose page has the link", 4L, webView)
        assertEquals(LINK, link.url)
        assertEquals("behind the page being read", true, link.background)
        assertEquals(true, link.user_gesture)
        assertEquals(0, menus())
        assertNull("and nothing is copied", clipboard.copied)
    }

    @Test
    fun copyLinkCopiesTheAddressAndAsksNothingOfTheCore() {
        val page = aWindow()
        press(page)

        compose.onNodeWithTag("link-copy").performClick()
        compose.waitForIdle()

        assertEquals(LINK, clipboard.copied)
        assertEquals(emptyList<Pair<Long, LinkOpenRequested>>(), asked())
        assertEquals(0, menus())
    }

    @Test
    fun cancelAndATouchBesideTheMenuPutItAwayAndDoNothing() {
        val page = aWindow()

        press(page)
        compose.onNodeWithTag("link-cancel").performClick()
        compose.waitForIdle()
        assertEquals(0, menus())

        press(page)
        // At the top of the page, where the menu at its foot is not.
        compose.onNodeWithTag("link-menu-beside").performTouchInput { click(topCenter + Offset(0f, 8f)) }
        compose.waitForIdle()
        assertEquals(0, menus())

        assertEquals(emptyList<Pair<Long, LinkOpenRequested>>(), asked())
        assertNull(clipboard.copied)
    }

    @Test
    fun theMenuIsOfThePageInFrontAndGoesWithIt() {
        val page = aWindow()

        // A page behind is not being touched: a press said of it shows nothing.
        press(pages.made.last())
        assertEquals(0, menus())

        // Another tab comes in front while the menu is open: the menu was not its page's.
        press(page)
        assertEquals(1, menus())
        theCoreSays(snapshot(url = "https://example.org/", display = "example.org", webView = 5))
        assertEquals(0, menus())
    }
}
