// The command bar (048): the address field's list has Spaces and commands in it. The window's
// model against a scripted core, and the window drawn by Compose on Robolectric's Android:
// what a question carries (the commands that can be done now, by their words, and that the
// window shows Spaces), what a Space's row and a command's row say, and what choosing one does.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.unit.LayoutDirection
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressSuggestions
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.Layout
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.Subscribed
import com.tigercodes.webtiger.core.v1.SuggestAddresses
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowOpened
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.core.v1.ZoomView
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
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
class CommandBarTest {
    @get:Rule val compose = createComposeRule()

    /** The rows the core offers back, whatever is typed. */
    private var rows: List<AddressSuggestion> = emptyList()

    private fun script(request: Request): Response =
        when {
            request.open_window != null -> Response(window_opened = WindowOpened(window_id = 1, tab_id = 2))
            request.subscribe != null -> Response(subscribed = Subscribed(subscription_id = 3))
            request.suggest_addresses != null -> Response(address_suggestions = AddressSuggestions(suggestions = rows))
            else -> Response(ack = Ack())
        }

    private val core = ScriptedCore(::script)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** The window as the core says it is: tab 2 in front, with a page or without. */
    private fun said(page: Boolean = true, change: (WindowSnapshot) -> WindowSnapshot = { it }): WindowSnapshot =
        change(
            WindowSnapshot(
                window_id = 1,
                tab =
                    TabView(
                        tab_id = 2,
                        web_view_id = if (page) 4 else 0,
                        address = if (page) AddressView(url = "https://example.com/", display = "example.com") else AddressView(),
                        zoom = if (page) ZoomView(percent = 100) else null,
                    ),
                tabs = listOf(TabSummary(tab_id = 2, host = if (page) "example.com" else "")),
            ),
        )

    private fun show(window: WindowSnapshot) {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        tell(window)
    }

    private fun tell(window: WindowSnapshot) {
        compose.runOnIdle { core.emit(window, subscription = 3) }
        compose.waitForIdle()
    }

    /**
     * Types in the address field, as the field tells the model of it, and gives the question
     * the core was then asked. (Robolectric's Android has no surface for the glass a text field
     * shows over what is typed: typed into the field itself a second time, it falls over.)
     */
    private fun type(text: String): SuggestAddresses {
        compose.runOnIdle {
            model.isEditingAddress = true
            model.addressText = text
        }
        compose.waitForIdle()
        return core.requests.last { it.suggest_addresses != null }.suggest_addresses!!
    }

    private fun space(id: Long, name: String) = AddressSuggestion(space_id = id, title = name)

    private fun command(name: String, words: String) = AddressSuggestion(command = name, title = words)

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun aQuestionCarriesTheCommandsThatCanBeDoneNowByTheirWords() {
        // An empty tab, alone, in a window with no room for a sidebar: a tab can be opened,
        // this one closed, and the lists shown.
        show(said(page = false))
        var asked = type("n")
        assertEquals(listOf("tab.new", "tab.close", "history.show"), asked.commands.map { it.name })
        assertEquals(listOf("New Tab", "Close Tab", "History and Downloads"), asked.commands.map { it.words })
        assertTrue("the window shows Spaces", asked.spaces)

        // A page: it can be kept, made a Favourite, found in and zoomed.
        tell(said())
        asked = type("ne")
        assertEquals(
            listOf("tab.new", "tab.close", "site.keep", "favourite.add", "page.find", "page.zoom", "history.show"),
            asked.commands.map { it.name },
        )

        // A Site that is a Favourite and has wandered, with a closed tab to bring back and a
        // tab beside it that it can change places with: each command says what it would do now.
        tell(
            said {
                it.copy(
                    tab = it.tab?.copy(kept = true, favourite = true, away = true),
                    undo = "tab.closed",
                    tabs = listOf(TabSummary(tab_id = 2, kept = true, favourite = true, steps_forward = true), TabSummary(tab_id = 5, kept = true)),
                )
            },
        )
        asked = type("new")
        assertEquals(
            listOf(
                "tab.new", "tab.reopen", "tab.close", "site.let_go", "favourite.remove", "site.home", "page.find", "page.zoom", "history.show",
                "tab.move_forward", "tab.next", "tab.previous",
            ),
            asked.commands.map { it.name },
        )
        assertEquals(
            listOf("Stop Keeping", "Remove from Favourites", "Return to Site", "Move Tab Right"),
            asked.commands.filter { it.name in setOf("site.let_go", "favourite.remove", "site.home", "tab.move_forward") }.map { it.words },
        )
    }

    @Test
    @Config(qualifiers = "w720dp-h900dp")
    fun aWindowWithRoomForASidebarOffersTheLayoutByWhereItsTabsAre() {
        show(said())
        var asked = type("s")
        assertEquals("Tabs at the Side", asked.commands.first { it.name == "layout.side" }.words)
        assertFalse(asked.commands.any { it.name == "layout.top" })
        // Down the side: the way back; and a tab that is no Favourite moves down, not right.
        tell(
            said {
                it.copy(layout = Layout.LAYOUT_SIDEBAR, tabs = listOf(TabSummary(tab_id = 2, host = "example.com", steps_forward = true), TabSummary(tab_id = 5)))
            },
        )
        asked = type("si")
        assertEquals("Tabs at the Top", asked.commands.first { it.name == "layout.top" }.words)
        assertFalse(asked.commands.any { it.name == "layout.side" })
        assertEquals("Move Tab Down", asked.commands.first { it.name == "tab.move_forward" }.words)
        // A Favourite's tile is in a row wherever the tabs are: it moves right, not down.
        tell(
            said {
                it.copy(
                    layout = Layout.LAYOUT_SIDEBAR,
                    tab = it.tab?.copy(kept = true, favourite = true),
                    tabs = listOf(TabSummary(tab_id = 2, kept = true, favourite = true, steps_forward = true), TabSummary(tab_id = 5, kept = true, favourite = true)),
                )
            },
        )
        asked = type("sid")
        assertEquals("Move Tab Right", asked.commands.first { it.name == "tab.move_forward" }.words)
    }

    @Test
    fun whereTheRowRunsFromRightToLeftTowardsItsEndIsLeft() {
        compose.setContent {
            CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Rtl) { WebTigerTheme { BrowserWindow(session, model) } }
        }
        compose.waitForIdle()
        tell(said { it.copy(tabs = listOf(TabSummary(tab_id = 2, host = "example.com", steps_forward = true), TabSummary(tab_id = 5))) })
        val asked = type("m")
        assertEquals("Move Tab Left", asked.commands.first { it.name == "tab.move_forward" }.words)
        assertFalse(asked.commands.any { it.name == "tab.move_back" })
    }

    @Test
    fun aPrivateWindowAsksForNoSpacesAndAWindowThatIsNotDrawnSendsNoCommands() =
        runTest {
            // Not drawn: the model has no words to send, and sends no commands.
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            assertEquals(0, model.barCommands().size)
            model.open()
            core.emit(said { it.copy(private_window = true) }, subscription = 3)
            model.isEditingAddress = true
            model.addressText = "w"
            val asked = core.requests.last { it.suggest_addresses != null }.suggest_addresses!!
            assertFalse("a private window has no Space", asked.spaces)
            assertEquals(emptyList<String>(), asked.commands.map { it.name })
            // What it could do, were it drawn: nothing of a private window is kept, so no
            // Favourite is offered.
            assertEquals(listOf("tab.new", "tab.close", "site.keep", "page.find", "page.zoom"), model.barCommands().map { it.name })
        }

    @Test
    fun aSpacesRowAndACommandsRowSayWhatTheyAreAndChosenAreDone() {
        rows = listOf(space(7, "Work"), command("tab.new", "New Tab"), command("site.keep", "Keep as Site"), command("no.such", "Nothing"))
        show(said())
        type("w")
        compose.onAllNodesWithTag("suggestion").assertCountEquals(4)
        // Each by its name, with what it is at its end, and no address under it.
        compose.onAllNodesWithTag("suggestion")[0].assertTextEquals("Work", "Space")
        compose.onAllNodesWithTag("suggestion")[1].assertTextEquals("New Tab", "Command")
        compose.onAllNodesWithTag("suggestion-switch", useUnmergedTree = true).assertCountEquals(4)

        // A Space: the core is asked to show it in this window, and the list is gone.
        compose.onAllNodesWithTag("suggestion")[0].performClick()
        compose.waitForIdle()
        val switch = core.requests.last().switch_space
        assertEquals(1L, switch?.window_id)
        assertEquals(7L, switch?.space_id)
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
        assertFalse(model.isEditingAddress)

        // A command: what its menu's item does.
        type("n")
        compose.onAllNodesWithTag("suggestion")[1].performClick()
        compose.waitForIdle()
        assertTrue(core.requests.last().open_tab != null)
        type("k")
        compose.onAllNodesWithTag("suggestion")[2].performClick()
        compose.waitForIdle()
        assertEquals(2L, core.requests.last().keep_tab?.tab_id)
        assertFalse(model.isEditingAddress)

        // A command the window did not send is done by nobody, and nothing is loaded for it.
        type("x")
        val before = core.requests.size
        compose.onAllNodesWithTag("suggestion")[3].performClick()
        compose.waitForIdle()
        assertEquals(before, core.requests.size)
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
    }

    @Test
    fun theListsAreShownByTheWindowAndFindAndZoomByTheModel() {
        rows = listOf(command("history.show", "History and Downloads"), command("page.find", "Find on Page"), command("page.zoom", "Zoom"))
        show(said())
        // Find and zoom are the model's: their bars open, one at a time.
        type("f")
        assertFalse(model.isFinding)
        compose.onAllNodesWithTag("suggestion")[1].performClick()
        compose.waitForIdle()
        assertTrue("the find bar is open", model.isFinding)
        type("z")
        compose.onAllNodesWithTag("suggestion")[2].performClick()
        compose.waitForIdle()
        assertTrue("the zoom bar is open", model.isZooming)
        // The lists of what was visited are the window's to show.
        type("h")
        compose.onAllNodesWithTag("history-done").assertCountEquals(0)
        compose.onAllNodesWithTag("suggestion")[0].performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("history-done").assertCountEquals(1)
    }

    @Test
    fun aRowSaysWhatItIsWhenItIsNoPlace() {
        assertEquals(null, kindOf(AddressSuggestion(url = "https://example.com/")))
        assertEquals(com.tigercodes.webtiger.R.string.suggestions_switch_tab, kindOf(AddressSuggestion(url = "https://example.com/", tab_id = 5)))
        assertEquals(com.tigercodes.webtiger.R.string.suggestions_space, kindOf(space(7, "Work")))
        assertEquals(com.tigercodes.webtiger.R.string.suggestions_command, kindOf(command("tab.new", "New Tab")))
    }
}
