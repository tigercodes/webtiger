// The adapter suite's System WebView run, for a search for words that are selected (023): the
// item the WebView's own bar for selected text has for it, chosen as a person's press would
// choose it. Who is told, with what words, and that nothing is started. No search is made:
// the adapter only reports, and there is no core here to open anything.
package com.tigercodes.webtiger.webview

import android.app.Instrumentation
import android.content.Intent
import android.content.IntentFilter
import android.view.ActionMode
import android.view.Window
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList

internal class SystemWebViewSearchTest : AdapterTestCase() {
    /** The bars the window has shown, in order: each with its items. */
    private val bars = CopyOnWriteArrayList<ActionMode>()

    /**
     * Whatever asks the system for a web search while a test runs is stopped here and counted:
     * were the adapter not to hear the WebView's request, nothing would start all the same.
     */
    private val websearches: Instrumentation.ActivityMonitor =
        InstrumentationRegistry.getInstrumentation().addMonitor(IntentFilter(Intent.ACTION_WEB_SEARCH), null, true)

    @After
    fun stopWatching() {
        InstrumentationRegistry.getInstrumentation().removeMonitor(websearches)
    }

    /** A web view on the page of links, shown in a window that says when a bar for selected text opens. */
    private fun page() {
        create(1)
        load(1, site.url("/links"))
        await(1, "the page to load") { finished(it) && hasTitle("Links")(it) }
        show(1)
        onStage { activity ->
            val window = activity.window
            val original = window.callback
            window.callback =
                object : Window.Callback by original {
                    override fun onActionModeStarted(mode: ActionMode) {
                        bars.add(mode)
                        original.onActionModeStarted(mode)
                    }
                }
        }
    }

    /** What the bar that is open offers, by its words. */
    private fun offered(): List<String> =
        onMain {
            val menu = bars.last().menu
            (0 until menu.size()).map { menu.getItem(it).title.toString() }
        }

    @Test
    fun theWebViewsSearchForSelectedWordsIsToldToTheCoreAndStartsNothing() {
        page()

        // A finger held on a word selects it, and the WebView shows its bar for selected text.
        longPress(1, "plain")
        await(1, "the bar for selected text to open") { bars.isNotEmpty() }
        val selected = run(1, "return String(window.getSelection());")
        assertEquals("the word under the finger is selected", "\"words\"", selected)
        // The WebView offers its search of the web: it can see something that answers one,
        // which is the app's own.
        assertTrue("a search of the web is offered among ${offered()}", offered().contains("Web search"))

        // That item, chosen as a press on it would choose it.
        onMain {
            val menu = bars.last().menu
            val item = (0 until menu.size()).map { menu.getItem(it) }.first { it.title.toString() == "Web search" }
            menu.performIdentifierAction(item.itemId, 0)
        }

        await(1, "the search to be told of") { reports -> reports.any { it.search_requested != null } }
        val asked = reports(1).mapNotNull { it.search_requested }
        assertEquals(1, asked.size)
        // The words the page says are selected, as the WebView names them.
        assertEquals("words", asked.first().text)
        assertEquals("the person chose it", true, asked.first().user_gesture)
        // Nothing was started: not the app's own answerer, and no other app.
        Thread.sleep(500)
        assertEquals(0, websearches.hits)
        // The page is where it was.
        assertEquals(listOf(site.url("/links")), addresses(1))
    }
}
