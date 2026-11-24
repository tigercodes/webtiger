// The Archive (049) over a scripted core: what its model asks and shows of the answers, what
// bringing a tab back and forgetting one send; the list drawn; and the window's way into it,
// which a private window has not.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.ArchiveEntry
import com.tigercodes.webtiger.core.v1.ArchivePage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.TabOpened
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import androidx.compose.ui.unit.dp
import java.time.ZoneOffset
import java.util.Locale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** What a scripted core has in its Archive: five tidied tabs, answered two at a time, newest first. */
internal class ScriptedArchive {
    private val kept = (5L downTo 1L).map { ArchiveEntry(id = it, title = "Page $it", url = "https://example.com/$it", display = "example.com/$it", tidied_at = 1_767_268_800_000 + it * 60_000) }.toMutableList()

    /** Another window took this one out of the Archive. */
    fun lose(id: Long) {
        kept.removeAll { it.id == id }
    }

    fun answer(request: Request): Response {
        request.query_archive?.let { query ->
            val older = kept.filter { query.before == 0L || it.id < query.before }
            return Response(archive_page = ArchivePage(entries = older.take(2), more = older.size > 2))
        }
        request.restore_archived?.let { restore ->
            if (!kept.removeAll { it.id == restore.entry_id }) throw CoreError.Refused(code = "archive.unknown", field = "entry_id")
            return Response(tab_opened = TabOpened(tab_id = 9))
        }
        request.forget_archived?.let { forget ->
            if (forget.all == true) kept.clear() else kept.removeAll { it.id == forget.entry_id }
            return Response(ack = Ack())
        }
        return windowScript(request)
    }
}

class ArchiveModelTest {
    private val archive = ScriptedArchive()

    private fun titles(model: ArchiveModel) = model.entries.map { it.title }

    /** What was asked of the core about the Archive, in order, in a few words each. */
    private fun asked(core: ScriptedCore): List<String> =
        core.requests.mapNotNull { request ->
            request.query_archive?.let { "list ${it.window_id} before ${it.before}" }
                ?: request.restore_archived?.let { "bring back ${it.entry_id} in ${it.window_id}" }
                ?: request.forget_archived?.let { if (it.all == true) "forget all in ${it.window_id}" else "forget ${it.entry_id} in ${it.window_id}" }
        }

    @Test
    fun theArchiveIsAskedOfTheWindowsSpaceNewestFirstAndOlderOnesAfterTheLastShown() =
        runTest {
            val core = ScriptedCore(archive::answer)
            val model = ArchiveModel(unconfinedSession(core), windowId = 7)
            assertFalse("nothing is said before the core has answered", model.loaded)

            model.reload()
            assertTrue(model.loaded)
            assertEquals(listOf("Page 5", "Page 4"), titles(model))
            assertTrue(model.more)

            model.showOlder()
            assertEquals(listOf("Page 5", "Page 4", "Page 3", "Page 2"), titles(model))
            model.showOlder()
            assertEquals(5, model.entries.size)
            assertFalse(model.more)
            assertEquals(listOf("list 7 before 0", "list 7 before 4", "list 7 before 2"), asked(core))
            // With no more, nothing is asked.
            model.showOlder()
            assertEquals(3, asked(core).size)
        }

    @Test
    fun aTabBroughtBackIsAskedOfTheCoreAndLeavesTheList() =
        runTest {
            val core = ScriptedCore(archive::answer)
            val model = ArchiveModel(unconfinedSession(core), windowId = 7)
            model.reload()
            var back = 0

            model.bringBack(model.entries[1]) { back += 1 }

            assertEquals("the core made it a tab again", 1, back)
            assertEquals(listOf("Page 5"), titles(model))
            // The model opens nothing by itself: the tab is the core's. Nothing else was asked.
            assertEquals(listOf("list 7 before 0", "bring back 4 in 7"), asked(core))
        }

    @Test
    fun aTabTheArchiveHasNoLongerIsNotBroughtBackAndTheListIsAskedForAgain() =
        runTest {
            val core = ScriptedCore(archive::answer)
            val model = ArchiveModel(unconfinedSession(core), windowId = 7)
            model.reload()
            val entry = model.entries[0]
            // Another window forgot it meanwhile.
            archive.lose(entry.id)
            var back = 0

            model.bringBack(entry) { back += 1 }

            assertEquals(0, back)
            assertEquals("what the Archive has now", listOf("Page 4", "Page 3"), titles(model))
            assertEquals(listOf("list 7 before 0", "bring back 5 in 7", "list 7 before 0"), asked(core))
        }

    @Test
    fun oneIsForgottenAndThenAllOfThem() =
        runTest {
            val core = ScriptedCore(archive::answer)
            val model = ArchiveModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.forget(model.entries[0])
            assertEquals("it leaves the list at once", listOf("Page 4"), titles(model))
            assertEquals("forget 5 in 7", asked(core).last())

            model.forgetEverything()
            assertEquals(emptyList<String>(), titles(model))
            assertFalse(model.more)
            assertEquals("and what is left is asked for", listOf("forget all in 7", "list 7 before 0"), asked(core).takeLast(2))
        }
}

@RunWith(RobolectricTestRunner::class)
class ArchiveListTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    @After
    fun end() {
        scope.cancel()
    }

    private fun session(core: ScriptedCore) = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)

    /** A core that opens a window, and has two tidied tabs: one with a name, one with none. */
    private var entries =
        listOf(
            ArchiveEntry(id = 2, title = "Example Domain", url = "https://example.com/", display = "example.com", tidied_at = 1_767_268_800_000),
            ArchiveEntry(id = 1, url = "https://example.net/plain.txt", display = "example.net/plain.txt", tidied_at = 1_767_268_000_000),
        )
    private val core =
        ScriptedCore { request ->
            when {
                request.query_archive != null -> Response(archive_page = ArchivePage(entries = entries, more = entries.isNotEmpty()))
                request.restore_archived != null -> Response(tab_opened = TabOpened(tab_id = 9))
                request.forget_archived?.all == true -> {
                    entries = emptyList()
                    Response(ack = Ack())
                }
                else -> windowScript(request)
            }
        }

    private var back = 0
    private var done = 0

    private fun show() {
        val model = ArchiveModel(session(core), windowId = 1)
        compose.setContent { WebTigerTheme { ArchiveList(model, onBroughtBack = { back += 1 }, onDone = { done += 1 }, zone = ZoneOffset.UTC, locale = Locale.UK) } }
        compose.waitForIdle()
    }

    @Test
    fun aTidiedTabIsShownByItsNameItsAddressAndWhenItWasTidied() {
        show()
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(2)
        val first = compose.onAllNodesWithTag("archive-entry")[0]
        first.assertTextContains("Example Domain")
        first.assertTextContains("example.com")
        first.assertTextContains("Tidied 1 Jan 2026, 12:00")
        // A page with no name is named by its address.
        compose.onAllNodesWithTag("archive-entry")[1].assertTextContains("example.net/plain.txt")
        compose.onNodeWithText("Archive").assertExists()
        compose.onNodeWithTag("archive-older").assertExists()
    }

    @Test
    fun aRowPressedIsBroughtBackAndDoneIsDone() {
        show()
        compose.onAllNodesWithTag("archive-entry")[0].performClick()
        compose.waitForIdle()
        val restore = core.requests.last().restore_archived
        assertEquals(2L, restore?.entry_id)
        assertEquals(1L, restore?.window_id)
        assertEquals("the tab is a tab again: the list's owner is told", 1, back)
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(1)
        compose.onNodeWithTag("archive-done").performClick()
        assertEquals(1, done)
    }

    @Test
    fun aTabIsForgottenByItsButton() {
        show()
        compose.onAllNodesWithTag("archive-forget").assertCountEquals(2)
        // Each says what it does to a screen reader.
        compose.onAllNodesWithContentDescription("Forget This Tab").assertCountEquals(2)
        compose.onAllNodesWithTag("archive-forget")[0].performClick()
        compose.waitForIdle()
        assertEquals(2L, core.requests.last().forget_archived?.entry_id)
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(1)
        assertEquals("nothing was brought back", 0, back)
    }

    @Test
    fun forgetAllAsksFirstAndCancelForgetsNothing() {
        show()
        compose.onNodeWithTag("archive-clear").performClick()
        compose.onNodeWithText("Forget every tab in this Archive?").assertExists()
        compose.onNodeWithText("Tabs that are forgotten can’t be brought back.").assertExists()
        // Nothing has been forgotten by asking.
        assertEquals(0, core.requests.count { it.forget_archived != null })
        compose.onNodeWithText("Cancel").performClick()
        compose.waitForIdle()
        assertEquals(0, core.requests.count { it.forget_archived != null })
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(2)

        compose.onNodeWithTag("archive-clear").performClick()
        compose.onNodeWithTag("archive-clear-everything").performClick()
        compose.waitForIdle()
        assertEquals(listOf(true), core.requests.mapNotNull { it.forget_archived }.map { it.all })
        assertNotNull("and what is left is asked for again", core.requests.last().query_archive)
        compose.onNodeWithText("Forget every tab in this Archive?").assertDoesNotExist()
        // Nothing is left: the list says so, says how a Space comes to be tidied, and has
        // nothing to forget.
        compose.onNodeWithTag("archive-nothing").assertTextContains("Tabs that are tidied away will be listed here.")
        compose.onNodeWithText("Nothing is tidied until you say so. To have this Space’s tabs tidied by themselves, edit the Space and choose after how long.").assertExists()
        compose.onNodeWithTag("archive-clear").assertIsNotEnabled()
    }

    /**
     * Narrower than the switch's four parts are together, whatever the words measure here (a
     * test on the JVM measures them as next to nothing: each part is as wide as its padding).
     */
    private val NARROW = 100.dp

    /** How much of a part of the switch is in sight: all of it, some of it, or none. */
    private fun shareInSight(tag: String): Float {
        val node = compose.onNodeWithTag(tag).fetchSemanticsNode()
        // What its row leaves of it, against what it is.
        return node.boundsInRoot.width / node.size.width
    }

    @Test
    fun whereTheNamesAreMoreThanTheRowIsWideTheOneShownIsBroughtInSight() {
        compose.setContent { WebTigerTheme { Box(Modifier.width(NARROW)) { LibrarySwitch(LibraryPart.ARCHIVE) {} } } }
        compose.waitForIdle()
        assertEquals("the one shown is all in sight", 1f, shareInSight("library-archive"), 0.01f)
        assertEquals("and the first is out of it", 0f, shareInSight("library-history"), 0.01f)
    }

    @Test
    fun theSwitchOpensAtItsStartWhereTheFirstListIsShown() {
        compose.setContent { WebTigerTheme { Box(Modifier.width(NARROW)) { LibrarySwitch(LibraryPart.HISTORY) {} } } }
        compose.waitForIdle()
        assertEquals(1f, shareInSight("library-history"), 0.01f)
        assertEquals("the last is past the row's end", 0f, shareInSight("library-archive"), 0.01f)
    }

    // The window's way into the list

    @Test
    fun theLibraryHasTheArchiveAndATabBroughtBackClosesIt() {
        val session = session(core)
        val model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.onNodeWithContentDescription("History and Downloads").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("library-archive").performClick()
        compose.waitForIdle()
        assertEquals("the window's own Archive is asked for", 1L, core.requests.last().query_archive?.window_id)
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(2)

        compose.onAllNodesWithTag("archive-entry")[0].performClick()
        compose.waitForIdle()

        assertEquals(2L, core.requests.last().restore_archived?.entry_id)
        // The model opened nothing by itself: no tab was asked for, and no address submitted.
        assertEquals(0, core.requests.count { it.open_tab != null || it.submit_address != null })
        compose.onAllNodesWithTag("archive-entry").assertCountEquals(0)
        compose.onAllNodesWithTag("library-archive").assertCountEquals(0)
    }
}
