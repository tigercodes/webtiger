// A private window on a phone (037 §5.3), drawn by Compose on Robolectric's Android over a
// scripted core: how one is asked for, what it says of itself, the page behind its chip, the
// row that opens it in the list of Spaces, and the holder that shows one window at a time.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.WindowHolder
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class PrivateWindowTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)

    @After
    fun end() {
        scope.cancel()
    }

    /** What the window did about the private window, in order. */
    private val did = ArrayList<String>()

    private fun ways(open: Boolean = false, other: String = "") =
        PrivateWindows(isOpen = open, other = other, show = { did.add("show") }, leave = { did.add("leave") }, close = { did.add("close") })

    private fun show(model: BrowserModel, private: PrivateWindows? = ways()) {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model, private) } }
        compose.waitForIdle()
    }

    /** Whether each window the core was asked to open was asked for as a private one. */
    private fun askedFor(core: ScriptedCore): List<Boolean> = core.requests.mapNotNull { it.open_window?.private_window }

    @Test
    fun aPrivateWindowIsAskedForAsOneAndIsOneFromTheStart() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val session = unconfinedSession(core)
            val ordinary = BrowserModel(session)
            val private = BrowserModel(session, asPrivate = true)
            // Before the core has said anything: the window knows what it was made as.
            assertFalse(ordinary.isPrivate)
            assertTrue(private.isPrivate)

            ordinary.open()
            private.open()

            assertEquals(listOf(false, true), askedFor(core))
        }

    @Test
    fun aWindowTheCoreSaysIsPrivateIsOne() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(WindowSnapshot(window_id = 1, tab = TabView(tab_id = 2)), subscription = 3)
            assertFalse(model.isPrivate)
            core.emit(WindowSnapshot(window_id = 1, tab = TabView(tab_id = 2), private_window = true), subscription = 3)
            assertTrue(model.isPrivate)
        }

    @Test
    fun aPrivateWindowSaysWhatItIsAndHasNoListOfSpaces() {
        val model = BrowserModel(session, asPrivate = true)
        show(model, ways(open = true, other = ""))

        // The chip: "Private" (or its first letter), and to a screen reader "Private Window".
        val chip = compose.onNodeWithTag("space-chip")
        assertEquals(listOf("Private Window"), chip.fetchSemanticsNode().config.getOrNull(SemanticsProperties.ContentDescription))
        // An empty tab says what private means, and what it does not.
        compose.onNodeWithTag("private-keeps", useUnmergedTree = true).assertTextEquals(
            "Nothing you visit in this window is kept on this device: no history, and nothing a site stored once the window is closed. Files you save stay.",
        )
        compose.onNodeWithTag("private-hides", useUnmergedTree = true).assertTextEquals("It does not hide you from the sites you visit, or from your network.")

        // The chip opens the page that says the same, not the list of Spaces: the core is
        // not asked for any.
        chip.performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("private-page").assertCountEquals(1)
        compose.onAllNodesWithTag("spaces").assertCountEquals(0)
        assertTrue(core.requests.none { it.query_spaces != null })
        // The other window is named by its Space; one nobody named is "Home".
        compose.onNodeWithText("Back to Home").assertExists()

        compose.onNodeWithTag("private-leave").performClick()
        compose.onNodeWithTag("private-close").performClick()
        assertEquals(listOf("leave", "close"), did)
        // Done leaves the window as it was.
        compose.onNodeWithTag("private-done").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("private-page").assertCountEquals(0)
        assertEquals(listOf("leave", "close"), did)
    }

    @Test
    fun anOrdinaryWindowsListOfSpacesEndsWithTheWayToThePrivateWindow() {
        val model = BrowserModel(session)
        show(model, ways(open = false))
        // An ordinary window says nothing of private, in its chip or its empty tab.
        compose.onAllNodesWithTag("private-keeps", useUnmergedTree = true).assertCountEquals(0)
        assertEquals(listOf("Space: Home"), compose.onNodeWithTag("space-chip").fetchSemanticsNode().config.getOrNull(SemanticsProperties.ContentDescription))

        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("spaces").assertCountEquals(1)
        compose.onNodeWithText("Private Window").assertExists()
        compose.onNodeWithText("Nothing it visits is kept on this device.").assertExists()
        // None is open: the row does not say one is.
        compose.onAllNodesWithTag("space-private-open", useUnmergedTree = true).assertCountEquals(0)

        compose.onNodeWithTag("space-private").performClick()
        compose.waitForIdle()

        // The private window is asked for, and the list is closed behind it.
        assertEquals(listOf("show"), did)
        compose.onAllNodesWithTag("spaces").assertCountEquals(0)
    }

    @Test
    fun theRowSaysWhenAPrivateWindowIsOpenAndIsNotThereWithoutAWayToOne() {
        val model = BrowserModel(session)
        show(model, ways(open = true))
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("space-private-open", useUnmergedTree = true).assertTextEquals("Open")
        assertEquals("Open", compose.onNodeWithTag("space-private").fetchSemanticsNode().config.getOrNull(SemanticsProperties.StateDescription))
    }

    @Test
    fun aWindowDrawnWithNoWayToAPrivateOneHasNoRowForIt() {
        // A picture of the window (004), and a test's: nothing to open a private window with.
        val model = BrowserModel(session)
        show(model, private = null)
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("spaces").assertCountEquals(1)
        compose.onAllNodesWithTag("space-private").assertCountEquals(0)
    }

    @Test
    fun theHolderShowsOneWindowAtATimeAndClosesThePrivateOneInTheCore() =
        runTest {
            val core = ScriptedCore { request ->
                // Each window the core opens has a number of its own: the private one is 9.
                if (request.open_window?.private_window == true) {
                    com.tigercodes.webtiger.core.v1.Response(window_opened = com.tigercodes.webtiger.core.v1.WindowOpened(window_id = 9, tab_id = 10))
                } else {
                    windowScript(request)
                }
            }
            val session = unconfinedSession(core)
            val holder = WindowHolder()
            val first = holder.model(session)
            assertSame(first, holder.shown(session))
            assertFalse(holder.hasPrivate)

            holder.showPrivate(session)
            val private = holder.shown(session)
            assertNotSame(first, private)
            assertTrue(private.isPrivate)
            assertTrue(holder.hasPrivate)
            // The first window is still the one that takes what other apps hand over.
            assertSame(first, holder.model(session))
            private.open()

            // Left: the first is shown, the private one stays, and is the same one when shown again.
            holder.leavePrivate()
            assertSame(first, holder.shown(session))
            assertTrue(holder.hasPrivate)
            holder.showPrivate(session)
            assertSame(private, holder.shown(session))
            assertTrue("nothing was closed by going back and forth", core.requests.none { it.close_window != null })

            // Closed: the core is asked to close that window, and no other; the first is shown.
            holder.closePrivate()
            assertSame(first, holder.shown(session))
            assertFalse(holder.hasPrivate)
            assertEquals(listOf(9L), core.requests.mapNotNull { it.close_window?.window_id })
            // The next private window is another one.
            holder.showPrivate(session)
            assertNotSame(private, holder.shown(session))
        }
}
