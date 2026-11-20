// The adapter suite's System WebView run, for a server that asks who is there (017): HTTP
// authentication. What the adapter reports, that the load waits for the core's word, what each
// word does, and that what the engine keeps of an answer stays in its login set.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.SignInAnswer
import com.tigercodes.webtiger.core.v1.SignInRequested
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewSignInTest : AdapterTestCase() {
    /** The questions the adapter has reported of a web view's servers, in order. */
    private fun asked(id: Long = 1): List<SignInRequested> = reports(id).mapNotNull { it.sign_in_requested }

    /** Waits until so many servers have asked in a web view, and gives the last. */
    private fun awaitAsked(id: Long = 1, count: Int = 1): SignInRequested {
        await(id, "a server to ask who is there ($count)") { asked(id).size >= count }
        return asked(id).last()
    }

    /** The core's word on a server's question: a name and a password. */
    private fun signIn(asked: SignInRequested, name: String, password: String) {
        assertNotNull(answer(HostRequest(sign_in_answer = SignInAnswer(request_id = asked.request_id, accepted = true, name = name, password = password))).ack)
    }

    /** The core's word that a server gets no answer. */
    private fun noAnswer(asked: SignInRequested) {
        assertNotNull(answer(HostRequest(sign_in_answer = SignInAnswer(request_id = asked.request_id, accepted = false))).ack)
    }

    private fun awaitPage(id: Long, title: String) {
        await(id, "the page \"$title\"; it last said \"${lastTitle(id)}\"") { finished(it) && hasTitle(title)(it) }
    }

    @Test
    fun aServerThatAsksIsReportedAndTheLoadWaitsForTheCoresWord() {
        create(1)
        load(1, site.url("/staff"))

        val asked = awaitAsked()

        assertEquals("the WebView names the server's host", "127.0.0.1", asked.host)
        assertEquals("and no origin", "", asked.origin)
        assertEquals("Staff", asked.realm)
        assertEquals("where the load is heading", site.url("/staff"), asked.heading)
        assertFalse("the first time", asked.again)
        assertNotEquals("named by the shell", 0L, asked.request_id)
        // The load waits: no page has come of it.
        Thread.sleep(500)
        assertEquals(emptyList<String>(), addresses(1))

        signIn(asked, "ada", "lovelace")

        awaitPage(1, "Staff only")
        assertEquals(listOf(site.url("/staff")), addresses(1))
        // The page's picture is the same server's: the engine answers for itself.
        Thread.sleep(500)
        assertEquals("asked once", 1, asked().size)
    }

    @Test
    fun aWrongAnswerAsksAgainAndSaysSo() {
        create(1)
        load(1, site.url("/staff"))
        val first = awaitAsked()

        signIn(first, "ada", "wrong")

        val second = awaitAsked(count = 2)
        assertTrue("the server refused an answer already", second.again)
        assertNotEquals("each has a name of its own", first.request_id, second.request_id)
        assertEquals("still the same load", site.url("/staff"), second.heading)
        signIn(second, "ada", "lovelace")
        awaitPage(1, "Staff only")
    }

    @Test
    fun noAnswerShowsWhatTheServerSentWithItsRefusal() {
        create(1)
        load(1, site.url("/staff"))

        noAnswer(awaitAsked())

        awaitPage(1, "Sign in needed")
        assertEquals("the refusal is a page, at the address asked for", listOf(site.url("/staff")), addresses(1))
        assertEquals("and nobody is asked again", 1, asked().size)
    }

    @Test
    fun whatTheEngineKeepsOfAnAnswerStaysInItsLoginSet() {
        val one = newLoginSet()
        create(1, loginSet = one)
        load(1, site.url("/staff"))
        signIn(awaitAsked(1), "ada", "lovelace")
        awaitPage(1, "Staff only")

        // Another web view of the same login set: the engine answers for itself.
        create(2, loginSet = one)
        load(2, site.url("/staff"))
        awaitPage(2, "Staff only")
        assertEquals("not asked again in the same login set", emptyList<SignInRequested>(), asked(2))

        // Another login set knows nothing of it: the server asks.
        create(3, loginSet = newLoginSet())
        load(3, site.url("/staff"))
        val again = awaitAsked(3)
        assertFalse("and it is that login set's first time", again.again)
        noAnswer(again)
        awaitPage(3, "Sign in needed")
    }

    @Test
    fun whereARedirectTookTheLoadIsWhereItIsHeading() {
        create(1)
        load(1, site.url("/to-staff?port=${site.port}"))

        val asked = awaitAsked()

        // The same server by its other name: where the load was sent on to.
        assertEquals("localhost", asked.host)
        assertEquals("http://localhost:${site.port}/staff", asked.heading)
        signIn(asked, "ada", "lovelace")
        awaitPage(1, "Staff only")
        assertEquals(listOf("http://localhost:${site.port}/staff"), addresses(1))
    }

    @Test
    fun aPictureOfThePagesOwnServerAsksWithNoLoadOnItsWay() {
        create(1)
        load(1, site.url("/shows-staff-picture?from=own"))

        val asked = awaitAsked()

        assertEquals("127.0.0.1", asked.host)
        assertEquals("the page has arrived: nothing is heading anywhere", "", asked.heading)
        signIn(asked, "ada", "lovelace")
        await(1, "the page to say its picture is shown; it last said \"${lastTitle(1)}\"") { hasTitle("picture shown")(it) }
    }

    @Test
    fun aPictureFromAnotherServerIsGivenNoAnswerByTheWebViewItself() {
        create(1)
        load(1, site.url("/shows-staff-picture"))

        // This WebView does not ask on behalf of a picture from another server: nothing is
        // reported, and the picture is refused. (An engine that did ask would be told the
        // same by the core's rule, 017 §3.)
        await(1, "the page to say its picture was refused; it last said \"${lastTitle(1)}\"") { hasTitle("picture refused")(it) }
        assertEquals(emptyList<SignInRequested>(), asked())
    }

    @Test
    fun aWebViewDestroyedWhileAServerWaits() {
        create(1)
        load(1, site.url("/staff"))
        val asked = awaitAsked()

        destroy(1)

        // A word for it afterwards is for nothing, and is acknowledged.
        signIn(asked, "ada", "lovelace")
        assertEquals(1, asked().size)
    }
}
