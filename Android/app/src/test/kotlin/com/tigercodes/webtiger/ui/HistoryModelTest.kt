// The History list's model (008) against a scripted core: what it asks, what it shows of the
// answers, and what it sends to have visits forgotten.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.core.v1.HistoryPage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** A core with six visits, which answers two at a time, and for words with one visit named after them. */
internal fun historyScript(request: Request): Response {
    val query = request.query_history ?: return Response(ack = Ack())
    val visits = (6L downTo 1L).map { HistoryEntry(visit_id = it, title = "Page $it", url = "https://example.com/$it", display = "example.com/$it") }
    if (query.text.isNotEmpty()) return Response(history_page = HistoryPage(entries = listOf(HistoryEntry(visit_id = 99, title = "Found by ${query.text}"))))
    val older = visits.filter { query.before_visit_id == 0L || it.visit_id < query.before_visit_id }
    return Response(history_page = HistoryPage(entries = older.take(2), more = older.size > 2))
}

class HistoryModelTest {
    private fun titles(model: HistoryModel) = model.entries.map { it.title }

    @Test
    fun itAsksForTheWindowsNewestVisitsAndShowsThem() =
        runTest {
            val core = ScriptedCore(::historyScript)
            val model = HistoryModel(unconfinedSession(core), windowId = 7)
            assertFalse("nothing is said before the core has answered", model.loaded)

            model.reload()

            val query = core.requests.last().query_history
            assertEquals(7L, query?.window_id)
            assertEquals("", query?.text)
            assertEquals("from the newest", 0L, query?.before_visit_id)
            assertTrue(model.loaded)
            assertEquals(listOf("Page 6", "Page 5"), titles(model))
            assertTrue(model.more)
        }

    @Test
    fun olderVisitsAreAskedForAfterTheLastOneShown() =
        runTest {
            val core = ScriptedCore(::historyScript)
            val model = HistoryModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.showOlder()

            assertEquals("older than the last one shown", 5L, core.requests.last().query_history?.before_visit_id)
            assertEquals(listOf("Page 6", "Page 5", "Page 4", "Page 3"), titles(model))
            model.showOlder()
            assertEquals(6, model.entries.size)
            assertFalse(model.more)
            // With no more, nothing is asked.
            val asked = core.requests.size
            model.showOlder()
            assertEquals(asked, core.requests.size)
        }

    @Test
    fun typingWordsAsksAgainAndTheSameWordsAskNothing() =
        runTest {
            val core = ScriptedCore(::historyScript)
            val model = HistoryModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.search("r")
            model.search("ru")
            model.search("rust")

            assertEquals(listOf("Found by rust"), titles(model))
            assertEquals(listOf("", "r", "ru", "rust"), core.requests.mapNotNull { it.query_history?.text })
            model.search("rust")
            assertEquals(4, core.requests.size)
        }

    @Test
    fun forgettingAVisitSendsItAndTakesItOffTheList() =
        runTest {
            val core = ScriptedCore(::historyScript)
            val model = HistoryModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.forget(model.entries[0])

            assertEquals("gone from the list at once", listOf("Page 5"), titles(model))
            val forget = core.requests.last().forget_history
            assertEquals(7L, forget?.window_id)
            assertEquals(6L, forget?.visit_id)
        }

    @Test
    fun forgettingManyAsksAgainForWhatIsLeft() =
        runTest {
            val core = ScriptedCore(::historyScript)
            val model = HistoryModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.forgetSince(1_767_268_800_500)
            model.forgetEverything()

            // Sent in the order made: each forgetting, then the question that follows it.
            val kinds =
                core.requests.drop(1).map { request ->
                    val forget = request.forget_history
                    when {
                        forget?.since_ms != null -> "forget since ${forget.since_ms}"
                        forget?.everything == true -> "forget everything"
                        request.query_history != null -> "ask"
                        else -> "other"
                    }
                }
            assertEquals(listOf("forget since 1767268800500", "ask", "forget everything", "ask"), kinds)
        }
}
