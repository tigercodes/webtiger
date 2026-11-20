// The adapter suite's System WebView run, for what a page says or asks (016): alert, confirm
// and prompt. What the adapter reports, that the page's script waits for the core's word, and
// what each word gives the page. The WebView shows no dialog of its own: the page moves on
// only when the test, as the core, has said how.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.DialogAnswer
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_ALERT
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_CONFIRM
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_PROMPT
import com.tigercodes.webtiger.core.v1.DialogRequested
import com.tigercodes.webtiger.core.v1.HostRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

internal class SystemWebViewDialogTest : AdapterTestCase() {
    /** A web view on the page that says and asks by its buttons. */
    private fun page(id: Long = 1, query: String = "") {
        create(id)
        load(id, site.url("/says$query"))
        await(id, "the page to load") { finished(it) && hasTitle("Says")(it) }
    }

    /** The dialogs the adapter has reported of a web view's pages, in order. */
    private fun said(id: Long = 1): List<DialogRequested> = reports(id).mapNotNull { it.dialog_requested }

    /** Taps one of the page's buttons with a finger, and waits for what it says to be reported. */
    private fun press(what: String, id: Long = 1): DialogRequested {
        val before = said(id).size
        tap(id, what)
        await(id, "the page to say something") { said(id).size > before }
        return said(id).last()
    }

    /** The core's word on a dialog. */
    private fun answer(dialog: DialogRequested, accepted: Boolean, text: String = "") {
        assertNotNull(answer(HostRequest(dialog_answer = DialogAnswer(request_id = dialog.request_id, accepted = accepted, text = text))).ack)
    }

    /** Waits for the page to say, in its title, what it got. */
    private fun awaitGot(words: String, id: Long = 1) {
        await(id, "the page to say \"$words\"; it last said \"${lastTitle(id)}\"") { hasTitle(words)(it) }
    }

    /** The page's own origin, as an origin is written. */
    private val origin: String get() = site.url("/").trimEnd('/')

    @Test
    fun anAlertIsReportedAndThePagesScriptWaitsForTheCoresWord() {
        page()

        val alert = press("alert")

        assertEquals(DIALOG_KIND_ALERT, alert.kind)
        assertEquals("Your basket is empty.", alert.message)
        assertEquals("who called: the page's origin, and nothing of its path", origin, alert.origin)
        assertNotEquals("named by the shell", 0L, alert.request_id)
        // The script waits: the line after the alert has not run.
        Thread.sleep(500)
        assertEquals("Says", lastTitle(1))

        answer(alert, accepted = true)

        awaitGot("alert returned")
    }

    @Test
    fun aConfirmIsTrueOrFalseAsTheCoreSays() {
        page()

        val first = press("confirm")
        assertEquals(DIALOG_KIND_CONFIRM, first.kind)
        assertEquals("Delete it?", first.message)
        answer(first, accepted = true)
        awaitGot("confirm true")

        val second = press("confirm")
        assertNotEquals("each has a name of its own", first.request_id, second.request_id)
        answer(second, accepted = false)
        awaitGot("confirm false")
    }

    @Test
    fun aPromptOffersItsTextAndGetsTheAnswerOrNothing() {
        page()

        val first = press("prompt")
        assertEquals(DIALOG_KIND_PROMPT, first.kind)
        assertEquals("Your name?", first.message)
        assertEquals("the text the page offers", "Ada", first.default_text)
        answer(first, accepted = true, text = "Grace Hopper")
        awaitGot("prompt Grace Hopper")

        answer(press("prompt"), accepted = false)
        awaitGot("prompt nothing")
    }

    @Test
    fun aFrameFromAnotherOriginIsReportedAsItself() {
        create(1)
        load(1, site.url("/says-in-a-frame?now=confirm"))

        await(1, "the frame to ask") { said().isNotEmpty() }

        // The frame is the same server by its other name: another origin than the page's.
        // The adapter says who called; whether a frame may is the core's rule (016 §3).
        val asked = said().last()
        assertEquals("http://localhost:${site.port}", asked.origin)
        assertEquals(DIALOG_KIND_CONFIRM, asked.kind)
        answer(asked, accepted = false)
        awaitGot("frame: confirm false")
    }

    @Test
    fun aPageThatAsksToKeepThePersonIsLeftAllTheSame() {
        page(query = "?hold=yes")

        // A finger on a link: the page asks to hold the person, and is not asked about.
        tap(1, "away")

        await(1, "the page it led to") { hasTitle("Landed")(it) }
        assertEquals("nothing was said of it", emptyList<DialogRequested>(), said())
    }

    @Test
    fun aWebViewDestroyedWhileItsPageWaitsTakesTheDialogWithIt() {
        page()
        val alert = press("alert")

        destroy(1)

        // A word for it afterwards is for nothing, and is acknowledged.
        answer(alert, accepted = true)
        assertEquals(1, said().size)
    }
}
