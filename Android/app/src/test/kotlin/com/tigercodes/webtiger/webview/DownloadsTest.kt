// The adapter's downloads (013) with no WebView and no system behind them: what is reported
// when a page offers one, what the fetcher is asked once the core has said, and what is said
// of how a fetch goes. Through the engine, as the core speaks to it.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.DownloadCancel
import com.tigercodes.webtiger.core.v1.DownloadStart
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.WebViewCreate
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val MINE = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val THEIRS = "7c2e4f6a-8b1d-4c3e-a5f7-9d0b2c4e6f8a"

/** A fetcher that fetches nothing: it writes down what it was asked, and says what a test tells it to. */
private class FakeFetcher : Fetcher {
    /** Every fetch started, in order: the address, the login set, the name. Its place in the list, from one, is its handle. */
    val started = ArrayList<Triple<String, String, String>>()
    val stopped = ArrayList<Long>()
    val statuses = HashMap<Long, FetchStatus?>()
    var starts = true

    override fun start(offer: DownloadOffer, loginSet: String, name: String): Long? {
        if (!starts) return null
        started.add(Triple(offer.url, loginSet, name))
        val handle = started.size.toLong()
        statuses[handle] = FetchStatus.Coming(0, offer.length)
        return handle
    }

    override fun stop(handle: Long) {
        stopped.add(handle)
    }

    override fun status(handle: Long): FetchStatus? = statuses[handle]
}

class DownloadsTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val fetcher = FakeFetcher()

    /** Whatever the engine would do a moment later is kept here, for a test to do when it likes. */
    private val later = ArrayList<() -> Unit>()
    /** Whoever asks after a file the person chose is kept waiting here, for a test to answer when it likes. */
    private val asks = ArrayList<Triple<String, String, String>>()
    private val answers = ArrayList<(Told) -> Unit>()
    private val asker =
        Asker { url, userAgent, loginSet, answer ->
            asks.add(Triple(url, userAgent, loginSet))
            answers.add(answer)
        }
    private val engine = SystemWebViewEngine(pages, fetcher, asker = asker) { later.add(it) }.also { it.attach(core) }

    private val notes = DownloadOffer(url = "https://example.com/notes", userAgent = "Agent/1", suggestedName = "notes.txt", mimeType = "text/plain", length = 26)

    private suspend fun aWebView(id: Long, loginSet: String = MINE): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = loginSet))))
        return pages.made.last()
    }

    private suspend fun start(download: Long, name: String) = engine.answer(HostRequest(download_start = DownloadStart(download_id = download, file_name = name)))

    private suspend fun cancel(download: Long) = engine.answer(HostRequest(download_cancel = DownloadCancel(download_id = download)))

    /** What the core was told of downloads, each in a few words. */
    private fun told(): List<String> =
        core.reports.mapNotNull { report ->
            val asked = report.download_requested
            val progress = report.download_progress
            val finished = report.download_finished
            val failed = report.download_failed
            when {
                asked != null -> "asked ${asked.download_id} by ${report.web_view_id}: ${asked.suggested_filename} ${asked.mime_type} ${asked.expected_bytes}"
                progress != null -> "progress ${progress.download_id}: ${progress.received_bytes}/${progress.expected_bytes}"
                finished != null -> "finished ${finished.download_id}: ${finished.file_name}"
                failed != null -> "failed ${failed.download_id}: ${failed.code}"
                else -> null
            }
        }

    @Test
    fun whatAPageOffersIsReportedAndNothingIsFetchedUntilTheCoreSays() =
        runTest {
            val page = aWebView(4)

            page.host.download(4, notes)

            assertEquals(listOf("asked 1 by 4: notes.txt text/plain 26"), told())
            assertEquals("https://example.com/notes", core.reports.last().download_requested?.url)
            assertEquals("a page offered it: nobody chose it in a menu (022)", false, core.reports.last().download_requested?.user_asked)
            assertTrue("nothing is fetched on a page's word", fetcher.started.isEmpty())

            assertNotNull(start(1, "My notes.txt").ack)

            assertEquals("the address, the login set whose cookies it is fetched with, and the core's name", listOf(Triple("https://example.com/notes", MINE, "My notes.txt")), fetcher.started)
        }

    @Test
    fun aFileThePersonChoseIsAskedAfterFirstAndOfferedAsWhereItIsAndWhatItsSiteSays() =
        runTest {
            val page = aWebView(4)

            engine.saveFile(4, "https://example.com/files/report")

            // Asked after, as the page's requests say who they are and in the page's login set;
            // nothing is said to the core until the answer.
            assertEquals(listOf(Triple("https://example.com/files/report", "Agent/1", MINE)), asks)
            assertEquals(listOf("https://example.com/files/report"), page.saved)
            assertEquals(emptyList<String>(), told())
            assertTrue("a page's own offer is not asked after", run { page.host.download(4, notes); asks.size == 1 })

            // The file is somewhere else, and its site says what it is.
            answers.single()(Told("https://cdn.example.net/2026/report.pdf", "application/pdf", "Report 2026.pdf", 4096))

            assertEquals(listOf("asked 1 by 4: notes.txt text/plain 26", "asked 2 by 4: Report 2026.pdf application/pdf 4096"), told())
            val offered = core.reports.last().download_requested
            assertEquals("where the file is, not where it was asked for", "https://cdn.example.net/2026/report.pdf", offered?.url)
            assertEquals("the person chose it", true, offered?.user_asked)
            // And that is the address the system is handed to fetch, with that site's cookies.
            assertNotNull(start(2, "Report 2026.pdf").ack)
            assertEquals(listOf(Triple("https://cdn.example.net/2026/report.pdf", MINE, "Report 2026.pdf")), fetcher.started)
        }

    @Test
    fun eachDownloadHasANameOfTheShellsOwnAndIsItsWebViewsLoginSets() =
        runTest {
            val mine = aWebView(4, MINE)
            val theirs = aWebView(5, THEIRS)
            mine.host.download(4, notes)
            theirs.host.download(5, notes.copy(url = "https://example.org/file", length = -1))

            assertEquals(listOf("asked 1 by 4: notes.txt text/plain 26", "asked 2 by 5: notes.txt text/plain 0"), told())
            start(2, "theirs.txt")
            start(1, "mine.txt")

            assertEquals(listOf(Triple("https://example.org/file", THEIRS, "theirs.txt"), Triple("https://example.com/notes", MINE, "mine.txt")), fetcher.started)
        }

    @Test
    fun howAFetchGoesIsReportedWhenItChanges() =
        runTest {
            val page = aWebView(4)
            page.host.download(4, notes)
            start(1, "notes.txt")
            core.reports.let { before -> assertEquals(1, before.size) }

            fetcher.statuses[1] = FetchStatus.Coming(10, 26)
            engine.pollDownloads()
            engine.pollDownloads()
            fetcher.statuses[1] = FetchStatus.Coming(20, 26)
            engine.pollDownloads()
            // The name was taken: the system says what it used.
            fetcher.statuses[1] = FetchStatus.Finished("notes-1.txt")
            engine.pollDownloads()
            engine.pollDownloads()

            assertEquals(
                "said once for each change, and once that it is whole",
                listOf("progress 1: 10/26", "progress 1: 20/26", "finished 1: notes-1.txt"),
                told().drop(1),
            )
        }

    @Test
    fun aFetchThatFailsIsReportedByItsCode() =
        runTest {
            val page = aWebView(4)
            page.host.download(4, notes)
            page.host.download(4, notes)
            start(1, "a.txt")
            start(2, "b.txt")

            fetcher.statuses[1] = FetchStatus.Failed("download.network")
            // The system no longer knows of the other: somebody removed it from under the app.
            fetcher.statuses[2] = null
            engine.pollDownloads()
            engine.pollDownloads()

            assertEquals(listOf("failed 1: download.network", "failed 2: download.cancelled"), told().drop(2))
        }

    @Test
    fun aFetchTheSystemWillNotStartIsADownloadThatFailed() =
        runTest {
            val page = aWebView(4)
            page.host.download(4, notes)
            fetcher.starts = false

            assertNotNull("the core's word was heard", start(1, "notes.txt").ack)

            assertEquals(listOf("failed 1: download.failed"), told().drop(1))
        }

    @Test
    fun aDownloadTheCoreDropsIsNeverFetchedAndOneThatIsComingIsStopped() =
        runTest {
            val page = aWebView(4)
            page.host.download(4, notes)
            page.host.download(4, notes)

            // One that was waiting for the core's word.
            assertNotNull(cancel(1).ack)
            assertEquals("download.unknown", start(1, "late.txt").error?.code)
            // One that is coming.
            start(2, "b.txt")
            assertNotNull(cancel(2).ack)
            fetcher.statuses[1] = FetchStatus.Finished("b.txt")
            engine.pollDownloads()

            assertEquals(listOf(Triple("https://example.com/notes", MINE, "b.txt")), fetcher.started)
            assertEquals("stopped, by the system's handle for it", listOf(1L), fetcher.stopped)
            assertEquals("and nothing more is said of either", 2, told().size)
            // A word about a download there is not.
            assertEquals("download.unknown", start(99, "nothing.txt").error?.code)
            assertNotNull(cancel(99).ack)
        }

    @Test
    fun theDownloadsAreAskedAfterForAsLongAsOneIsComing() =
        runTest {
            val page = aWebView(4)
            page.host.download(4, notes)
            assertTrue("nothing is coming: nothing to ask after", later.isEmpty())

            start(1, "notes.txt")
            assertEquals("a look is on its way", 1, later.size)
            fetcher.statuses[1] = FetchStatus.Coming(10, 26)
            later.removeAt(0)()
            assertEquals(listOf("progress 1: 10/26"), told().drop(1))
            assertEquals("and another after it", 1, later.size)

            fetcher.statuses[1] = FetchStatus.Finished("notes.txt")
            later.removeAt(0)()
            assertEquals("finished 1: notes.txt", told().last())
            assertTrue("with nothing coming, the looking stops", later.isEmpty())
        }
}
