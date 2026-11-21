// The Downloads list drawn (013), over a scripted core: what it shows, and what a person's
// choices send. And the window's way into it: the switch beside History, and the dot that says
// a download is coming.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.DownloadEntry
import com.tigercodes.webtiger.core.v1.DownloadState
import com.tigercodes.webtiger.core.v1.DownloadsPage
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.HistoryPage
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class DownloadsListTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    @After
    fun end() {
        scope.cancel()
    }

    private fun session(core: ScriptedCore) = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)

    private fun download(id: Long, name: String, state: DownloadState, received: Long = 0, total: Long = 0, code: String = "", host: String = "example.com") =
        DownloadEntry(id = id, file_name = name, host = host, state = state, received_bytes = received, total_bytes = total, error_code = code)

    /** What the core has, newest first: one coming, one that finished, one that failed. A test changes it. */
    private var downloads =
        listOf(
            download(3, "report.pdf", DownloadState.DOWNLOAD_STATE_COMING, received = 1_000_000, total = 4_000_000),
            download(2, "notes.txt", DownloadState.DOWNLOAD_STATE_FINISHED, total = 2_000),
            download(1, "photos.zip", DownloadState.DOWNLOAD_STATE_FAILED, code = "download.network", host = "example.net"),
        )

    /** A core that opens a window, lists these downloads, forgets what it is told to, and has visited nothing. */
    private val core =
        ScriptedCore { request ->
            val forget = request.forget_downloads
            if (forget != null) downloads = if (forget.everything == true) emptyList() else downloads.filter { it.id != forget.download_id }
            val refused = notOpened
            when {
                request.open_download != null && refused != null -> Response(error = Error(code = refused))
                request.query_downloads != null -> Response(downloads_page = DownloadsPage(downloads = downloads))
                request.query_history != null -> Response(history_page = HistoryPage())
                else -> windowScript(request)
            }
        }

    private var done = 0

    private fun show(watching: Boolean = false) {
        val model = DownloadsModel(session(core), windowId = 1)
        compose.setContent { WebTigerTheme { DownloadsList(model, onDone = { done += 1 }, watching = watching) } }
        compose.waitForIdle()
    }

    private fun asked(): Int = core.requests.count { it.query_downloads != null }

    /** What the core says when it is asked to open a download: nothing, when it opens. */
    private var notOpened: String? = null

    /** The downloads the core was asked to open, in order. */
    private fun opened(): List<Long> = core.requests.mapNotNull { it.open_download?.download_id }

    @Test
    fun aDownloadThatHasFinishedIsPressedToOpenItsFileAndTheOthersAreNot() {
        show()
        val entries = compose.onAllNodesWithTag("download-entry")

        // Only the one with a whole file can be pressed, and a screen reader says what pressing does.
        entries[1].assert(SemanticsMatcher.expectValue(SemanticsProperties.Role, Role.Button))
        val label = entries[1].fetchSemanticsNode().config.getOrNull(SemanticsActions.OnClick)?.label
        assertEquals("Open", label)
        for (other in listOf(0, 2)) {
            assertEquals("one with no whole file is not pressed", null, entries[other].fetchSemanticsNode().config.getOrNull(SemanticsActions.OnClick))
        }

        entries[1].performClick()
        compose.waitForIdle()

        assertEquals(listOf(2L), opened())
        compose.onAllNodesWithTag("download-not-opened", useUnmergedTree = true).assertCountEquals(0)
    }

    @Test
    fun whyAFileDidNotOpenIsSaidUnderItInWords() {
        notOpened = "download.missing"
        show()
        val finished = compose.onAllNodesWithTag("download-entry")[1]

        finished.performClick()
        compose.waitForIdle()
        finished.assertTextContains("The file isn’t where it was saved any more.")

        notOpened = "system.no_handler"
        finished.performClick()
        compose.waitForIdle()
        finished.assertTextContains("Nothing on this device opens this kind of file.")

        // A code this build has no words for says something, and never the code.
        notOpened = "download.something_new"
        finished.performClick()
        compose.waitForIdle()
        finished.assertTextContains("The file couldn’t be opened.")
        compose.onAllNodesWithTag("download-not-opened", useUnmergedTree = true).assertCountEquals(1)

        // Pressed again and opened: nothing is said.
        notOpened = null
        finished.performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("download-not-opened", useUnmergedTree = true).assertCountEquals(0)
        assertEquals(4, opened().size)
    }

    @Test
    fun aDownloadIsShownByItsNameItsSiteAndHowItStands() {
        show()
        compose.onAllNodesWithTag("download-entry").assertCountEquals(3)
        val coming = compose.onAllNodesWithTag("download-entry")[0]
        coming.assertTextContains("report.pdf")
        coming.assertTextContains("example.com")
        coming.assertTextContains("1.0 MB of 4.0 MB")
        // One that finished says how big it is; one that failed says why, in words.
        compose.onAllNodesWithTag("download-entry")[1].assertTextContains("2.0 kB")
        val failed = compose.onAllNodesWithTag("download-entry")[2]
        failed.assertTextContains("example.net")
        failed.assertTextContains("The connection was lost.")
        // One that is coming can be stopped; the others can be taken off the list.
        compose.onAllNodesWithTag("download-stop").assertCountEquals(1)
        compose.onAllNodesWithTag("download-forget").assertCountEquals(2)
        compose.onNodeWithContentDescription("Stop Download").assertExists()
    }

    @Test
    fun howADownloadStandsIsSaidInWords() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        // Sizes are written as the system writes them: in thousands, since Android 8.
        assertEquals("1.0 MB of 4.0 MB", standing(context, downloads[0]))
        assertEquals("no size was said: how much so far", "500 kB so far", standing(context, download(9, "a", DownloadState.DOWNLOAD_STATE_COMING, received = 500_000)))
        assertEquals("2.0 kB", standing(context, downloads[1]))
        assertEquals("a file of no known size says nothing", "", standing(context, download(9, "a", DownloadState.DOWNLOAD_STATE_FINISHED)))
        for ((code, words) in listOf(
            "download.cancelled" to "Stopped",
            "download.network" to "The connection was lost.",
            "download.disk" to "The file couldn’t be saved.",
            "download.interrupted" to "WebTiger was closed before it finished.",
            "download.failed" to "The download failed.",
            // A code this build has no words for says something, and never itself.
            "download.new_reason" to "The download failed.",
            "" to "The download failed.",
        )) {
            assertEquals(code, words, standing(context, download(9, "a", DownloadState.DOWNLOAD_STATE_FAILED, code = code)))
        }
    }

    @Test
    fun stoppingAsksTheCoreAndForgettingTakesItOffTheList() {
        show()
        compose.onNodeWithTag("download-stop").performClick()
        compose.waitForIdle()
        assertEquals(listOf(3L), core.requests.mapNotNull { it.cancel_download?.download_id })
        assertNotNull("and the list is asked for again", core.requests.last().query_downloads)

        compose.onAllNodesWithTag("download-forget")[0].performClick()
        compose.waitForIdle()

        assertEquals(2L, core.requests.last().forget_downloads?.download_id)
        compose.onAllNodesWithTag("download-entry").assertCountEquals(2)
    }

    @Test
    fun clearTakesEverythingOffAndThenThereIsNothingToClear() {
        show()
        compose.onNodeWithTag("downloads-nothing").assertDoesNotExist()
        compose.onNodeWithTag("downloads-clear").assertIsEnabled().performClick()
        compose.waitForIdle()

        assertEquals(listOf(true), core.requests.mapNotNull { it.forget_downloads }.map { it.everything })
        compose.onAllNodesWithTag("download-entry").assertCountEquals(0)
        compose.onNodeWithTag("downloads-nothing").assertTextContains("Files you download will be listed here.")
        compose.onNodeWithTag("downloads-clear").assertIsNotEnabled()
        compose.onNodeWithTag("downloads-done").performClick()
        assertEquals(1, done)
    }

    @Test
    fun whileTheListIsShownItIsAskedForAgainEverySecond() {
        compose.mainClock.autoAdvance = false
        show(watching = true)
        val first = asked()
        assertEquals("asked when it is shown", 1, first)

        compose.mainClock.advanceTimeBy(1_050)
        compose.waitForIdle()
        assertEquals(2, asked())
        // What the core says now is what the list shows: the download has got further.
        downloads = listOf(download(3, "report.pdf", DownloadState.DOWNLOAD_STATE_COMING, received = 2_000_000, total = 4_000_000))
        compose.mainClock.advanceTimeBy(1_050)
        compose.waitForIdle()
        assertEquals(3, asked())
        compose.onAllNodesWithTag("download-entry")[0].assertTextContains("2.0 MB of 4.0 MB")
    }

    @Test
    fun aPictureOfTheListAsksOnce() {
        compose.mainClock.autoAdvance = false
        show(watching = false)
        compose.mainClock.advanceTimeBy(3_000)
        compose.waitForIdle()
        assertEquals(1, asked())
    }

    // The window's way into the list

    @Test
    fun theSwitchBesideHistoryShowsTheDownloadsAndBack() {
        val session = session(core)
        val model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()

        compose.onNodeWithContentDescription("History and Downloads").performClick()
        compose.waitForIdle()
        // History first, as before; the switch says which is shown.
        compose.onNodeWithTag("history-search").assertExists()
        compose.onNodeWithTag("library-history").assertIsSelected()
        compose.onNodeWithTag("library-downloads").assertIsNotSelected()
        assertEquals("nothing is asked about downloads until they are shown", 0, asked())

        compose.onNodeWithTag("library-downloads").performClick()
        compose.waitForIdle()

        assertEquals("the window's own downloads are asked for", 1L, core.requests.last { it.query_downloads != null }.query_downloads?.window_id)
        compose.onAllNodesWithTag("download-entry").assertCountEquals(3)
        compose.onNodeWithTag("library-downloads").assertIsSelected()
        compose.onNodeWithTag("history-search").assertDoesNotExist()

        compose.onNodeWithTag("library-history").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("history-search").assertExists()
        compose.onAllNodesWithTag("download-entry").assertCountEquals(0)
        compose.onNodeWithTag("history-done").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("library-history").assertDoesNotExist()
    }

    @Test
    fun aDotOnTheButtonSaysADownloadIsComing() {
        val session = session(core)
        val model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.onNodeWithTag("downloading", useUnmergedTree = true).assertDoesNotExist()

        core.emit(snapshot(url = "https://example.com/", display = "example.com", webView = 4).copy(downloads_active = 1), subscription = 3)
        compose.waitForIdle()

        compose.onNodeWithTag("downloading", useUnmergedTree = true).assertExists()
        assertEquals(1, model.downloadsActive)

        core.emit(snapshot(url = "https://example.com/", display = "example.com", webView = 4), subscription = 3)
        compose.waitForIdle()
        compose.onNodeWithTag("downloading", useUnmergedTree = true).assertDoesNotExist()
    }
}
