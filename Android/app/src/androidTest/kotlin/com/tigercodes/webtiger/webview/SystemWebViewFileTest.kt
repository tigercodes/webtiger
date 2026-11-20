// The adapter suite's System WebView run, for a page that asks for a file (019): a finger on
// the test site's file inputs, and a chooser of the test's own that picks files the test wrote.
// The system's picker is never started.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.util.concurrent.CopyOnWriteArrayList

internal class SystemWebViewFileTest : AdapterTestCase() {
    /** What the chooser was asked, in order; and what it picks next. */
    private val asked = CopyOnWriteArrayList<FilesAsked>()

    @Volatile private var picks: List<String> = emptyList()

    private val folder = File(ApplicationProvider.getApplicationContext<Context>().cacheDir, "webtiger-upload-${System.nanoTime()}")

    override fun fileChooser(): FileChooser =
        FileChooser { wanted, done ->
            asked.add(wanted)
            done(picks)
        }

    @After
    fun tidy() {
        folder.deleteRecursively()
    }

    /** A file of the test's own, with these words in it: its address, as a chooser gives one. */
    private fun file(name: String, words: String): String {
        folder.mkdirs()
        val made = File(folder, name)
        made.writeText(words)
        return Uri.fromFile(made).toString()
    }

    /** A web view on the page of file inputs. */
    private fun page() {
        create(1)
        load(1, site.url("/upload"))
        await(1, "the page to load") { finished(it) && hasTitle("Upload")(it) }
    }

    private fun awaitTitle(title: String) {
        await(1, "the page to say \"$title\"; it last said \"${lastTitle(1)}\"") { hasTitle(title)(it) }
    }

    @Test
    fun aPressOnAFileInputAsksTheChooserAndThePageHasTheFile() {
        picks = listOf(file("notes.txt", "Twelve notes, to be sent."))
        page()

        tap(1, "one")

        awaitTitle("chosen: notes.txt (25): Twelve notes, to be sent.")
        assertEquals("asked once", 1, asked.size)
        assertFalse("for one file", asked.single().several)
    }

    @Test
    fun severalFilesWhereSeveralMayBeChosenAndTheKindsThePageAccepts() {
        picks = listOf(file("a.txt", "first"), file("b.txt", "second file"))
        page()

        tap(1, "many")

        awaitTitle("chosen: a.txt (5), b.txt (11): first")
        assertTrue(asked.single().several)
        assertEquals("what the page says it accepts, as the WebView passes it on", listOf("text/plain", ".txt"), asked.single().kinds)
    }

    @Test
    fun nothingPickedAndThePageHasNothingAndMayAskAgain() {
        page()

        tap(1, "one")
        awaitTitle("cancelled one")

        picks = listOf(file("notes.txt", "Now."))
        tap(1, "one")
        awaitTitle("chosen: notes.txt (4): Now.")
        assertEquals(2, asked.size)
    }
}
