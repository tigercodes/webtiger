package com.tigercodes.webtiger.ui

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** The words for the core's codes, from the app's own resources (generated from Localization/strings.json). */
@RunWith(RobolectricTestRunner::class)
class WordsTest {
    private val context: Context = ApplicationProvider.getApplicationContext()

    private fun words(code: String) = errorWords(context.resources, context.packageName, code)

    @Test
    fun everyErrorCodeHasWords() {
        assertEquals("The site’s address couldn’t be found.", words("net.dns"))
        assertEquals("You’re not connected to the internet.", words("net.offline"))
        assertEquals("The site couldn’t be reached.", words("net.connect"))
        assertEquals("The site took too long to answer.", words("net.timeout"))
        assertEquals("A secure connection to the site couldn’t be made.", words("net.tls"))
        assertEquals("The page stopped unexpectedly.", words("page.renderer_gone"))
        assertEquals("This page’s engine isn’t available.", words("engine.unavailable"))
    }

    @Test
    fun aCodeWithoutWordsOfItsOwnStillSaysSomething() {
        val general = context.getString(R.string.error_net_failed)
        assertEquals("Something went wrong while loading the page.", general)
        // A code this build has no words for, and never the code's own text.
        assertEquals(general, words("net.something_new"))
        // The adapter's own refusals have words of their own.
        assertNotEquals(general, words("engine.no_profiles"))
        assertNotEquals(general, words("engine.no_private"))
        // Not codes at all: they name no string, not even one that exists ("error.title").
        assertEquals(general, words("title"))
        assertEquals(general, words(""))
        assertEquals(general, words("Net.DNS"))
        assertNotEquals(context.getString(R.string.error_title), words("title"))
    }

    @Test
    fun theStartFailureNamesItsCode() {
        assertEquals("Error code: store.open", context.getString(R.string.error_start_detail, "store.open"))
    }
}
