// The context the adapter's WebViews are made with (023): a request to search the web is the
// adapter's to hear, and starts nothing; every other request goes on as it was.
package com.tigercodes.webtiger.webview

import android.app.Application
import android.app.SearchManager
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf

@RunWith(RobolectricTestRunner::class)
class SearchingContextTest {
    private val application: Application = ApplicationProvider.getApplicationContext()
    private val searched = ArrayList<String>()
    private val context = SearchingContext(application) { searched.add(it) }

    /** What the WebView asks the system for when its search for `words` is chosen. */
    private fun search(words: String?): Intent =
        Intent(Intent.ACTION_WEB_SEARCH).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK).apply { if (words != null) putExtra(SearchManager.QUERY, words) }

    @Test
    fun aSearchOfTheWebIsHeardAndStartsNothing() {
        context.startActivity(search("tigers in the wild"))
        // Asked each way an app can ask: with something said of how to start it, or nothing.
        context.startActivity(search("lions"), null)
        context.startActivity(search("leopards"), Bundle())

        assertEquals(listOf("tigers in the wild", "lions", "leopards"), searched)
        assertNull("no other app was started", shadowOf(application).nextStartedActivity)
    }

    @Test
    fun aSearchWithNoWordsIsHeardAsNone() {
        context.startActivity(search(null))

        assertEquals(listOf(""), searched)
        assertNull(shadowOf(application).nextStartedActivity)
    }

    @Test
    fun everyOtherRequestGoesOnAsItWas() {
        val letter = Intent(Intent.ACTION_VIEW, Uri.parse("mailto:ada@example.com"))
        // A search asked for in words of another kind is no web search.
        val within = Intent(Intent.ACTION_SEARCH)
        // Words for some app to do something with.
        val words = Intent(Intent.ACTION_PROCESS_TEXT).setType("text/plain").putExtra(Intent.EXTRA_PROCESS_TEXT, "tigers")

        for (asked in listOf(letter, within, words)) {
            context.startActivity(asked.putExtra(SearchManager.QUERY, "tigers").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            assertEquals(asked.action, shadowOf(application).nextStartedActivity?.action)
        }
        assertEquals(emptyList<String>(), searched)
    }
}
