// What the System WebView gives a tab that is put to sleep and woken (000 §5.7, spike Q5): the
// web view is destroyed, and a new one in the same profile is handed what `saveState` wrote.
// Sleeping tabs are W3; these tests pin the platform behaviour that design rests on, so a
// WebView that changes it fails here first. The WebKit half is WebKitRestoreTests.swift.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.os.Bundle
import android.util.Log
import android.view.View
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.webkit.WebViewCompat
import com.tigercodes.webtiger.TestSite
import java.util.UUID
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewRestoreTest {
    private val site = TestSite()
    private val loginSet = UUID.randomUUID().toString()
    private val made = ArrayList<WebView>()

    @After
    fun release() {
        onMain { made.forEach { it.destroy() } }
        site.close()
    }

    // Helpers

    private fun <T> onMain(block: () -> T): T {
        var result: Result<T>? = null
        InstrumentationRegistry.getInstrumentation().runOnMainSync { result = runCatching(block) }
        return checkNotNull(result).getOrThrow()
    }

    /** A web view in the login set's profile, the size of a small window, in no window at all. */
    private fun makeWebView(): WebView =
        onMain {
            WebView(ApplicationProvider.getApplicationContext<Context>()).also { view ->
                WebViewCompat.setProfile(view, loginSet)
                view.settings.javaScriptEnabled = true
                view.webViewClient = WebViewClient()
                view.measure(View.MeasureSpec.makeMeasureSpec(800, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(600, View.MeasureSpec.EXACTLY))
                view.layout(0, 0, 800, 600)
                made.add(view)
            }
        }

    private fun destroy(view: WebView) {
        onMain { view.destroy() }
        made.remove(view)
    }

    /** Evaluates an expression in the page; what it gives, as JSON. */
    private fun evaluate(view: WebView, script: String): String {
        val answer = CompletableFuture<String?>()
        onMain { view.evaluateJavascript(script) { json -> answer.complete(json) } }
        return answer.get(10, TimeUnit.SECONDS).orEmpty()
    }

    /** Waits until the web view's document is `path` and has finished loading. */
    private fun awaitPage(view: WebView, path: String) {
        val wanted = "\"complete ${site.url(path)}\""
        repeat(1500) {
            if (evaluate(view, "document.readyState + ' ' + location.href") == wanted) return
            Thread.sleep(10)
        }
        throw AssertionError("the web view never showed $path; it shows ${evaluate(view, "location.href")}")
    }

    private fun load(view: WebView, path: String) {
        onMain { view.loadUrl(site.url(path)) }
        awaitPage(view, path)
    }

    /** What `saveState` writes: the tab's state while it has no web view. */
    private fun sleep(view: WebView): Bundle {
        val state = Bundle()
        assertNotNull("the web view's state could be saved", onMain { view.saveState(state) })
        destroy(view)
        return state
    }

    private fun wake(state: Bundle): WebView {
        val woken = makeWebView()
        assertNotNull("a new web view took the state", onMain { woken.restoreState(state) })
        return woken
    }

    // Tests

    @Test
    fun aSleepingTabsStateIsSmallAndRestoresItsHistoryInANewWebView() {
        val awake = makeWebView()
        load(awake, "/")
        load(awake, "/landed")
        load(awake, "/tall")
        val asleep = sleep(awake)

        // The state can be kept: it is one array of bytes, and small.
        assertEquals("the state is one thing, under one key", setOf("WEBVIEW_CHROMIUM_STATE"), asleep.keySet())
        val bytes = checkNotNull(asleep.getByteArray("WEBVIEW_CHROMIUM_STATE"))
        assertTrue("000 §7: a sleeping tab is at most 250 KB on disk", bytes.size < 250_000)
        Log.i("WTFACT", "System WebView saved state for three pages of history: ${bytes.size} bytes")

        // Woken from the bytes alone, as a tab is that slept through a restart of the app.
        val woken = wake(Bundle().apply { putByteArray("WEBVIEW_CHROMIUM_STATE", bytes) })
        awaitPage(woken, "/tall")
        assertEquals("Tall", onMain { woken.title })
        assertTrue("the history came back with it", onMain { woken.canGoBack() })
        assertFalse(onMain { woken.canGoForward() })
        val history = onMain { woken.copyBackForwardList().let { list -> (0 until list.size).map { list.getItemAtIndex(it).url } } }
        assertEquals(listOf(site.url("/"), site.url("/landed"), site.url("/tall")), history)

        onMain { woken.goBack() }
        awaitPage(woken, "/landed")
        assertTrue(onMain { woken.canGoForward() })
    }

    @Test
    fun aWokenTabKeepsItsLoginSet() {
        val awake = makeWebView()
        load(awake, "/set?v=ada")
        val woken = wake(sleep(awake))
        awaitPage(woken, "/set?v=ada")
        load(woken, "/whoami")
        assertEquals("waking a tab does not sign it out", "who=ada", onMain { woken.title })
    }

    /**
     * The WebView's saved state carries the scroll position and the text typed into a form,
     * once the engine has brought its history entry up to date: about a second after the page
     * last changed. Put to sleep 0.6 s after the scroll and the typing, the tab woke at the top
     * with an empty field; after 2 s it woke as it was left. A tab sleeps after minutes in the
     * background (000 §5.7), so the second has long passed; under memory pressure it may not have.
     * (WebKit differs: it keeps the scroll position and never the text.)
     */
    @Test
    fun aWokenTabIsWhereItWasScrolledToAndKeepsTheTextInItsForm() {
        val awake = makeWebView()
        load(awake, "/tall")
        val typed =
            evaluate(
                awake,
                """
                (() => {
                    const note = document.getElementById('note');
                    note.focus();
                    document.execCommand('insertText', false, 'a draft');
                    window.scrollTo(0, 2400);
                    return note.value + ' ' + window.scrollY;
                })()
                """.trimIndent(),
            )
        assertEquals("the field took the text and the page scrolled", "\"a draft 2400\"", typed)
        Thread.sleep(2_000)

        val woken = wake(sleep(awake))
        awaitPage(woken, "/tall")
        var after = ""
        for (attempt in 0 until 50) {
            after = evaluate(woken, "document.getElementById('note').value + '|' + Math.round(window.scrollY)")
            if (after == "\"a draft|2400\"") break
            Thread.sleep(100)
        }
        assertEquals("a woken tab is where it was left, with what was typed in it", "\"a draft|2400\"", after)
    }
}
