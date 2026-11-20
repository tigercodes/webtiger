// What a page says or asks (016), with no WebView behind the adapter: what is reported, that
// the page waits for the core's word, and what each word gives the page. Through the engine,
// as the core speaks to it.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.DialogAnswer
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_ALERT
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_CONFIRM
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_PROMPT
import com.tigercodes.webtiger.core.v1.DialogRequested
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val PAGE = "https://example.com/basket?item=7#top"

class DialogsTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val engine = SystemWebViewEngine(pages).also { it.attach(core) }

    /** What each page was answered, in order: the web view, whether it was accepted, and the text. */
    private val told = ArrayList<Triple<Long, Boolean, String>>()

    private suspend fun aWebView(id: Long): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    /** A page says something or asks, as its web view would say it. */
    private fun say(page: FakePage, kind: DialogKind, message: String, defaultText: String = "", url: String = PAGE) {
        page.host.dialog(page.id, url, kind, message, defaultText) { accepted, text -> told.add(Triple(page.id, accepted, text)) }
    }

    private suspend fun answer(request: Long, accepted: Boolean, text: String = "") =
        engine.answer(HostRequest(dialog_answer = DialogAnswer(request_id = request, accepted = accepted, text = text)))

    /** The dialogs the core was told of, in order. */
    private fun reported(): List<DialogRequested> = core.reports.mapNotNull { it.dialog_requested }

    @Test
    fun whatAPageSaysIsReportedWithWhoCalledAndThePageWaits() =
        runTest {
            val page = aWebView(4)

            say(page, DIALOG_KIND_PROMPT, "Your name?", "Ada")

            val asked = reported().single()
            assertEquals(4L, core.reports.last().web_view_id)
            assertEquals("who called, and nothing of its path", "https://example.com", asked.origin)
            assertEquals(DIALOG_KIND_PROMPT, asked.kind)
            assertEquals("Your name?", asked.message)
            assertEquals("Ada", asked.default_text)
            assertTrue("named by the shell", asked.request_id != 0L)
            assertTrue("the page's script waits for the core's word", told.isEmpty())
        }

    @Test
    fun theCoresWordIsThePagesAnswer() =
        runTest {
            val page = aWebView(4)
            say(page, DIALOG_KIND_ALERT, "Saved.")
            say(page, DIALOG_KIND_CONFIRM, "Delete it?")
            say(page, DIALOG_KIND_PROMPT, "Your name?", "Ada")
            val (alert, confirm, prompt) = reported().map { it.request_id }

            assertNotNull(answer(prompt, accepted = true, text = "Grace").ack)
            answer(confirm, accepted = false)
            answer(alert, accepted = true)

            assertEquals(listOf(Triple(4L, true, "Grace"), Triple(4L, false, ""), Triple(4L, true, "")), told)
        }

    @Test
    fun eachDialogHasANameOfItsOwnAndIsAnsweredOnce() =
        runTest {
            val first = aWebView(4)
            val second = aWebView(5)
            say(first, DIALOG_KIND_ALERT, "One")
            say(second, DIALOG_KIND_ALERT, "Two")
            val names = reported().map { it.request_id }
            assertEquals(2, names.toSet().size)

            answer(names[0], accepted = false)
            answer(names[0], accepted = true)
            assertEquals("the first word stands", listOf(Triple(4L, false, "")), told)

            // A name that was answered is not given again.
            say(first, DIALOG_KIND_ALERT, "Three")
            assertTrue(reported().last().request_id !in names)
            // A word about a dialog there is not is acknowledged, and is for nothing.
            assertNotNull(answer(99, accepted = true).ack)
            assertEquals(1, told.size)
        }

    @Test
    fun aWebViewThatIsDestroyedDismissesWhatItsPagesSaid() =
        runTest {
            val going = aWebView(4)
            val staying = aWebView(5)
            say(going, DIALOG_KIND_CONFIRM, "Leave?")
            say(going, DIALOG_KIND_ALERT, "And another thing")
            say(staying, DIALOG_KIND_CONFIRM, "Stay?")
            val names = reported().map { it.request_id }

            engine.answer(HostRequest(web_view_destroy = WebViewDestroy(web_view_id = 4)))

            assertEquals("both of its own, as if the person had closed them", listOf(Triple(4L, false, ""), Triple(4L, false, "")), told)
            // A word for one of them afterwards is for nothing; another web view's still waits.
            answer(names[0], accepted = true)
            answer(names[2], accepted = true)
            assertEquals(Triple(5L, true, ""), told.last())
            assertEquals(3, told.size)
        }

    @Test
    fun whoCalledIsWrittenAsAnOrigin() {
        for ((url, origin) in listOf(
            "https://example.com/a/b?c=d#e" to "https://example.com",
            "https://example.com" to "https://example.com",
            "https://example.com:8443/x" to "https://example.com:8443",
            "http://127.0.0.1:8080/says" to "http://127.0.0.1:8080",
            "HTTP://Example.com/" to "http://Example.com",
            // An address that is none of the web's has no origin: the core dismisses it.
            "about:blank" to "",
            "data:text/html,hello" to "",
            "file:///sdcard/page.html" to "",
            "" to "",
            "not an address at all" to "",
        )) {
            assertEquals(url, origin, Dialogs.originOf(url))
        }
    }
}
