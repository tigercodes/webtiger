// "Search with WebTiger" in other apps' menu of selected text (028): what the entry reads of
// what another app hands it, what it starts, what the window then asks of the core, and that
// the system is told of the entry. No search is made: the core here is a script.
package com.tigercodes.webtiger

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.text.SpannableString
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.unconfinedSession
import com.tigercodes.webtiger.ui.windowScript
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf

@RunWith(RobolectricTestRunner::class)
class SearchWordsTest {
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** What another app's menu of selected text hands the item it was chosen from. */
    private fun selected(words: CharSequence?, action: String = Intent.ACTION_PROCESS_TEXT): Intent =
        Intent(action).setType("text/plain").apply { if (words != null) putExtra(Intent.EXTRA_PROCESS_TEXT, words) }

    /** The words the core was asked to search for, with the window they were for, in order. */
    private fun searched(core: ScriptedCore): List<Pair<Long, String>> = core.requests.mapNotNull { it.search_words?.let { asked -> asked.window_id to asked.text } }

    @Test
    fun theWordsAreReadOnlyFromTheSystemsOwnRequestAndOnlyWhenThereAreSome() {
        assertEquals("tigers in the wild", SearchWordsActivity.words(selected("tigers in the wild")))
        // Words with a style, as a selection can have: only the words.
        assertEquals("tigers", SearchWordsActivity.words(selected(SpannableString("tigers"))))
        // As they are: tidying them is the core's.
        assertEquals("  tigers \n in the wild ", SearchWordsActivity.words(selected("  tigers \n in the wild ")))

        assertNull("no words", SearchWordsActivity.words(selected(null)))
        assertNull(SearchWordsActivity.words(selected("")))
        assertNull(SearchWordsActivity.words(selected(" \n\t ")))
        assertNull(SearchWordsActivity.words(null))
        // Anything else that carries the same words is not this item being chosen.
        for (other in listOf(Intent.ACTION_SEND, Intent.ACTION_VIEW, Intent.ACTION_MAIN, Intent.ACTION_WEB_SEARCH, MainActivity.ACTION_SEARCH_WORDS)) {
            assertNull(other, SearchWordsActivity.words(selected("tigers", action = other)))
        }
        // A whole document is not carried: what is handed on is kept small.
        assertEquals(SearchWordsActivity.MOST_CARRIED, SearchWordsActivity.words(selected("w".repeat(50_000)))?.length)
    }

    @Test
    fun chosenInAnotherAppItStartsTheWindowWithTheWordsAndIsGone() {
        val activity = Robolectric.buildActivity(SearchWordsActivity::class.java, selected("tigers in the wild")).create().get()

        val started = shadowOf(activity).nextStartedActivity
        assertEquals(ComponentName(context, MainActivity::class.java), started.component)
        assertEquals(MainActivity.ACTION_SEARCH_WORDS, started.action)
        assertEquals("tigers in the wild", started.getStringExtra(MainActivity.EXTRA_WORDS))
        // In the window's own task, not in the task of the app the words came from.
        assertEquals(Intent.FLAG_ACTIVITY_NEW_TASK, started.flags and Intent.FLAG_ACTIVITY_NEW_TASK)
        // No address rides along: the words are all there is.
        assertNull(started.data)
        assertNull("only one thing is started", shadowOf(activity).nextStartedActivity)
        assertTrue("it shows nothing, and goes", activity.isFinishing)
        // Nothing is handed back to the app the words were selected in.
        assertEquals(android.app.Activity.RESULT_CANCELED, shadowOf(activity).resultCode)
        assertNull(shadowOf(activity).resultIntent)
    }

    @Test
    fun startedWithNoWordsOrSomeOtherWayItStartsNothingAndIsGone() {
        for (intent in listOf(selected(null), selected("  "), selected("tigers", action = Intent.ACTION_SEND), Intent(Intent.ACTION_MAIN))) {
            val activity = Robolectric.buildActivity(SearchWordsActivity::class.java, intent).create().get()

            assertNull(shadowOf(activity).nextStartedActivity)
            assertTrue(activity.isFinishing)
        }
    }

    @Test
    fun theWindowHandsTheWordsToTheCoreWithItsWindowAndMakesNothingOfThem() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()

            // What looks like an address, and what would be run if anything ran words: handed
            // on as they are. That they are only ever searched for is the core's rule.
            for (words in listOf("tigers in the wild", "example.com", "javascript:alert(1)")) {
                MainActivity.searchWords(SearchWordsActivity.windowIntent(context, words), model)
            }

            assertEquals(listOf(1L to "tigers in the wild", 1L to "example.com", 1L to "javascript:alert(1)"), searched(core))
            // Nothing else was asked for them: no tab opened by the shell, nothing loaded.
            assertTrue(core.requests.none { it.open_tab != null || it.submit_address != null })
        }

    @Test
    fun anIntentThatCarriesNoWordsAsksTheCoreNothing() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()

            MainActivity.searchWords(null, model)
            MainActivity.searchWords(Intent(Intent.ACTION_MAIN), model)
            MainActivity.searchWords(Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_WORDS, "tigers"), model)
            MainActivity.searchWords(Intent(MainActivity.ACTION_SEARCH_WORDS), model)
            MainActivity.searchWords(Intent(MainActivity.ACTION_SEARCH_WORDS).putExtra(MainActivity.EXTRA_WORDS, "  "), model)

            assertEquals(emptyList<Pair<Long, String>>(), searched(core))
        }

    @Test
    fun wordsThatComeWithTheStartOfTheAppWaitForTheWindow() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))

            model.searchFor("first thoughts")
            model.searchFor("tigers in the wild")
            assertEquals("there is no window to search in yet", emptyList<Pair<Long, String>>(), searched(core))

            model.open()

            // The last words asked for, once, in the window that opened.
            assertEquals(listOf(1L to "tigers in the wild"), searched(core))
            model.searchFor("lions")
            assertEquals(listOf(1L to "tigers in the wild", 1L to "lions"), searched(core))

            // A window that is closed and opened again does not search for them a second time.
            model.close()
            model.open()
            assertEquals(listOf(1L to "tigers in the wild", 1L to "lions"), searched(core))
        }

    @Test
    fun theSystemIsToldOfTheItemAndOfNothingElseThatTakesText() {
        val asked = Intent(Intent.ACTION_PROCESS_TEXT).setType("text/plain")

        val answers = context.packageManager.queryIntentActivities(asked, 0).map { it.activityInfo }

        assertEquals(listOf(SearchWordsActivity::class.java.name), answers.map { it.name })
        val item = answers.single()
        assertTrue("other apps can start it: that is what it is for", item.exported)
        assertEquals("Search with WebTiger", item.loadLabel(context.packageManager).toString())
        // It is never part of the window's task, and is not in the list of recent apps.
        assertEquals("", item.taskAffinity.orEmpty())
        assertTrue(item.flags and android.content.pm.ActivityInfo.FLAG_EXCLUDE_FROM_RECENTS != 0)
        assertTrue(item.flags and android.content.pm.ActivityInfo.FLAG_NO_HISTORY != 0)
        // The window is one, however it is started: the one there is comes forward and is told.
        val window = context.packageManager.getActivityInfo(ComponentName(context, MainActivity::class.java), 0)
        assertTrue(window.exported)
        assertEquals(android.content.pm.ActivityInfo.LAUNCH_SINGLE_TASK, window.launchMode)
        // And it answers nothing of another app's but its own launch.
        assertFalse(context.packageManager.queryIntentActivities(Intent(Intent.ACTION_VIEW).setData(android.net.Uri.parse("https://example.com/")), 0).any { it.activityInfo.packageName == context.packageName })
        assertFalse(context.packageManager.queryIntentActivities(Intent(Intent.ACTION_SEND).setType("text/plain"), 0).any { it.activityInfo.packageName == context.packageName })
    }
}
