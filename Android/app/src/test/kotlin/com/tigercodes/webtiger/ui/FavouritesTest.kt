// Favourites (042): the few Sites that are always first. The window's model against a scripted
// core, and the strip drawn by Compose on Robolectric's Android: the tile, what it shows and
// says, and the command in a tab's own menu.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.TabSummary
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
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class FavouritesTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun tab(
        id: Long,
        favourite: Boolean = false,
        kept: Boolean = favourite,
        asleep: Boolean = false,
        away: Boolean = false,
        title: String = "Tab $id",
        host: String = "example.com",
        siteName: String = "",
    ): TabSummary = TabSummary(tab_id = id, title = title, host = host, kept = kept, favourite = favourite, asleep = asleep, away = away, site_name = siteName)

    /** The window as the core says it is: tab 2 in front with a page, these tabs, and whether the row is full. */
    private fun said(tabs: List<TabSummary>, full: Boolean = false, private: Boolean = false): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "Tab 2", address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = tabs,
            favourites_full = full,
            private_window = private,
        )

    /** What a core was asked about Favourites and about closing, in order: "add 2", "remove 5", "close 5". */
    private fun asked(core: ScriptedCore = this.core): List<String> =
        core.requests.mapNotNull { request ->
            request.set_favourite?.let { "${if (it.favourite) "add" else "remove"} ${it.tab_id}" }
                ?: request.close_tab?.let { "close ${it.tab_id}" }
        }

    @Test
    fun aTabIsMadeAFavouriteOrStopsBeingOneByAskingTheCoreAndNoMoreIsOfferedWhenTheRowIsFull() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(listOf(tab(2), tab(5, favourite = true), tab(6, title = "", host = ""))), subscription = 3)
            // A tab with a page can be made one; one that is can stop being one; an empty tab
            // has nothing to keep.
            assertTrue(model.canFavourite(model.tabs[0]))
            assertTrue(model.canFavourite(model.tabs[1]))
            assertFalse(model.canFavourite(model.tabs[2]))
            model.setFavourite(2, true)
            model.setFavourite(5, false)
            assertEquals(listOf("add 2", "remove 5"), asked(core))

            // The row is full, by the core's word: no more can be made one; one that is can
            // still stop being one. With room again, a tab can.
            core.emit(said(listOf(tab(2), tab(5, favourite = true)), full = true), subscription = 3)
            assertTrue(model.favouritesFull)
            assertFalse(model.canFavourite(model.tabs[0]))
            assertTrue(model.canFavourite(model.tabs[1]))
            core.emit(said(listOf(tab(2), tab(5, favourite = true))), subscription = 3)
            assertTrue(model.canFavourite(model.tabs[0]))

            // A private window has no Favourites (037).
            core.emit(said(listOf(tab(2)), private = true), subscription = 3)
            assertFalse(model.canFavourite(model.tabs[0]))
        }

    /** A window whose strip has two Favourites (the second asleep, kept as "mail" and wandered once), a Site, and the tab in front. */
    private fun show(full: Boolean = false) {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                said(
                    listOf(
                        tab(5, favourite = true, away = true, title = "Re: lunch", host = "mail.example.com", siteName = "mail"),
                        tab(6, favourite = true, asleep = true, title = "Board", siteName = "Board"),
                        tab(7, kept = true, title = "Wiki"),
                        tab(2, title = "News"),
                    ),
                    full = full,
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    private fun property(tag: String, key: androidx.compose.ui.semantics.SemanticsPropertyKey<String>): List<String> =
        compose.onAllNodesWithTag(tag).fetchSemanticsNodes().map { it.config.getOrNull(key).orEmpty() }

    @Test
    fun aFavouriteIsATileWithALetterThatSaysItsWholeNameAndHasNoButtonAtItsEnd() {
        show()
        compose.onAllNodesWithTag("tab").assertCountEquals(4)
        // Two tiles: the first letter of the name each was kept with, as a capital, not of
        // where it has wandered to.
        val letters =
            compose.onAllNodesWithTag("favourite-letter", useUnmergedTree = true).fetchSemanticsNodes().map { node ->
                node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString { it.text }
            }
        assertEquals(listOf("M", "B"), letters)
        // A tile says its whole name, and what it is: a Favourite is a Site, and is not said
        // to be both.
        assertEquals(
            listOf("Re: lunch", "Board"),
            compose.onAllNodesWithTag("tab").fetchSemanticsNodes().take(2).map { it.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty().joinToString() },
        )
        assertEquals(listOf("Favourite, Away", "Favourite, Asleep", "Site", ""), property("tab", SemanticsProperties.StateDescription))
        // A tile has no mark of a Site, and no button at its end: the Site and the tab have theirs.
        compose.onAllNodesWithTag("site-mark", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("close-tab").assertCountEquals(2)
        // A press brings it in front, as any tab.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performClick()
        assertEquals(6L, core.requests.last().activate_tab?.tab_id)
    }

    @Test
    fun aTabsOwnMenuMakesItAFavouriteOrStopsItBeingOneAndPutsATileToSleep() {
        show()
        // A Favourite that is awake, held: put to sleep from here, there being no button at
        // its end; and the command to stop being one.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("favourite").assertTextEquals("Remove from Favourites")
        compose.onNodeWithTag("favourite-sleep").assertTextEquals("Put to Sleep")
        compose.onNodeWithTag("favourite-sleep").performClick()
        compose.waitForIdle()
        assertEquals(listOf("close 5"), asked())
        compose.onAllNodesWithTag("favourite").assertCountEquals(0)

        // One that is asleep has no page to put to sleep; it stops being a Favourite.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("favourite-sleep").assertCountEquals(0)
        compose.onNodeWithTag("favourite").performClick()
        compose.waitForIdle()
        assertEquals(listOf("close 5", "remove 6"), asked())

        // A Site, and a tab, are made one; neither has "Put to Sleep" in its menu.
        compose.onAllNodesWithTag("tab")[2].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("favourite").assertTextEquals("Add to Favourites")
        compose.onNodeWithTag("favourite").assertIsEnabled()
        compose.onAllNodesWithTag("favourite-sleep").assertCountEquals(0)
        compose.onNodeWithTag("favourite").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("tab")[3].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("favourite").performClick()
        compose.waitForIdle()
        assertEquals(listOf("close 5", "remove 6", "add 7", "add 2"), asked())
    }

    @Test
    fun whenTheCoreSaysTheRowIsFullTheCommandIsGreyForATabThatIsNone() {
        show(full = true)
        compose.onAllNodesWithTag("tab")[3].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("favourite").assertTextEquals("Add to Favourites")
        compose.onNodeWithTag("favourite").assertIsNotEnabled()
        compose.onNodeWithTag("favourite").performClick()
        compose.waitForIdle()
        assertEquals("nothing is asked", emptyList<String>(), asked())
        // "Keep as Site" is not the Favourites' to refuse.
        compose.onNodeWithTag("keep-or-let-go").assertIsEnabled()
        compose.onNodeWithTag("keep-or-let-go").performClick()
        compose.waitForIdle()
        // One that is a Favourite can still stop being one.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("favourite").assertIsEnabled()
    }
}
