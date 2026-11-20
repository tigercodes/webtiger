// The Downloads list's model (013) against a scripted core: what it asks, what it shows of the
// answers, and what it sends to have a download stopped or taken off the list.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.DownloadEntry
import com.tigercodes.webtiger.core.v1.DownloadState
import com.tigercodes.webtiger.core.v1.DownloadsPage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * A core with downloads, newest first, which answers two at a time unless it is asked for
 * another number, and takes off its list what it is told to forget. The newest is coming.
 */
internal class DownloadsScript(count: Long = 6) {
    var downloads: List<DownloadEntry> =
        (count downTo 1L).map {
            DownloadEntry(
                id = it,
                file_name = "file-$it.txt",
                host = "example.com",
                state = if (it == count) DownloadState.DOWNLOAD_STATE_COMING else DownloadState.DOWNLOAD_STATE_FINISHED,
                total_bytes = 2_048,
                received_bytes = if (it == count) 1_024 else 0,
            )
        }

    fun answer(request: Request): Response {
        val forget = request.forget_downloads
        if (forget != null) {
            downloads = if (forget.everything == true) emptyList() else downloads.filter { it.id != forget.download_id }
        }
        val query = request.query_downloads ?: return Response(ack = Ack())
        val older = downloads.filter { query.before_id == 0L || it.id < query.before_id }
        val limit = if (query.limit == 0) 2 else query.limit
        return Response(downloads_page = DownloadsPage(downloads = older.take(limit), more = older.size > limit))
    }
}

class DownloadsModelTest {
    private fun names(model: DownloadsModel) = model.entries.map { it.file_name }

    @Test
    fun theListIsWhatTheCoreSaysNewestFirstAndOlderWhenAskedFor() =
        runTest {
            val core = ScriptedCore(DownloadsScript()::answer)
            val model = DownloadsModel(unconfinedSession(core), windowId = 7)
            assertFalse("nothing is said before the core has answered", model.loaded)

            model.reload()

            val query = core.requests.last().query_downloads
            assertEquals(7L, query?.window_id)
            assertEquals("from the newest", 0L, query?.before_id)
            assertTrue(model.loaded)
            assertEquals(listOf("file-6.txt", "file-5.txt"), names(model))
            assertTrue(model.more)
            assertTrue("the newest is coming", model.isComing)

            model.showOlder()

            assertEquals("older than the last one shown", 5L, core.requests.last().query_downloads?.before_id)
            assertEquals(listOf("file-6.txt", "file-5.txt", "file-4.txt", "file-3.txt"), names(model))
            model.showOlder()
            assertEquals(6, model.entries.size)
            assertFalse(model.more)
            // With no more, nothing is asked.
            val asked = core.requests.size
            model.showOlder()
            assertEquals(asked, core.requests.size)
        }

    @Test
    fun stoppingAsksTheCoreAndTheListIsAskedForAgain() =
        runTest {
            val core = ScriptedCore(DownloadsScript()::answer)
            val model = DownloadsModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.stop(model.entries[0])

            val sent = core.requests.takeLast(2)
            assertEquals("the download is named by the core's own id", 6L, sent[0].cancel_download?.download_id)
            assertNotNull("and what the core then says of it is asked for", sent[1].query_downloads)
        }

    @Test
    fun forgettingOneLeavesTheListAtOnceAndForgettingAllAsksAgain() =
        runTest {
            val script = DownloadsScript()
            val core = ScriptedCore(script::answer)
            val model = DownloadsModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.forget(model.entries[1])

            val forget = core.requests.last().forget_downloads
            assertEquals(7L, forget?.window_id)
            assertEquals(5L, forget?.download_id)
            assertEquals("it has left, with nothing asked", listOf("file-6.txt"), names(model))

            model.forgetEverything()

            val sent = core.requests.takeLast(2)
            assertEquals(true, sent[0].forget_downloads?.everything)
            assertNotNull("what is left is asked for", sent[1].query_downloads)
            assertTrue(model.entries.isEmpty())
            assertTrue("the core has answered: nothing was downloaded", model.loaded)
            assertFalse(model.isComing)
        }

    @Test
    fun aListTakenFurtherDownIsAskedForWholeAgain() =
        runTest {
            // More than the core gives when it is not told how many: a hundred and fifty, a
            // hundred at a time.
            val script = DownloadsScript(count = 150)
            val core =
                ScriptedCore { request ->
                    val query = request.query_downloads
                    script.answer(if (query != null && query.limit == 0) request.copy(query_downloads = query.copy(limit = 100)) else request)
                }
            val model = DownloadsModel(unconfinedSession(core), windowId = 7)
            model.reload()
            assertEquals("a first question leaves the number to the core", 0, core.requests.last().query_downloads?.limit)
            model.showOlder()
            assertEquals(150, model.entries.size)

            model.reload()

            val query = core.requests.last().query_downloads
            assertEquals("from the newest", 0L, query?.before_id)
            assertEquals("and as far down as the list is shown", 150, query?.limit)
            assertEquals(150, model.entries.size)
        }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun onlyTheAnswerToTheLastQuestionIsShown() =
        runTest {
            val script = DownloadsScript()
            val core = LateAnswers(ScriptedCore(script::answer))
            val session = BrowserSession(core, SystemWebViewEngine(FakePages()), CoroutineScope(backgroundScope.coroutineContext + UnconfinedTestDispatcher(testScheduler)))
            val model = DownloadsModel(session, windowId = 7)

            // The first question's answer is made, and held; by the second, the newest has gone.
            model.reload()
            script.downloads = script.downloads.drop(1)
            model.reload()
            assertEquals(2, core.held.size)
            core.held[1].complete(Unit)
            assertEquals(listOf("file-5.txt", "file-4.txt"), names(model))
            core.held[0].complete(Unit)

            assertEquals("an answer to a question that was asked again is out of date", listOf("file-5.txt", "file-4.txt"), names(model))
        }
}

/** A core whose answer about downloads waits until the test lets it go: answers that come in any order. */
private class LateAnswers(private val core: ScriptedCore) : CoreClient by core {
    val held = ArrayList<CompletableDeferred<Unit>>()

    override suspend fun request(request: Request): Response {
        val answer = core.request(request)
        if (request.query_downloads != null) CompletableDeferred<Unit>().also { held.add(it) }.await()
        return answer
    }
}
