// A server that asks who is there (017), with no WebView behind the adapter: what is reported,
// that the load waits for the core's word, and what each word gives the engine. Through the
// engine, as the core speaks to it.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.SignInAnswer
import com.tigercodes.webtiger.core.v1.SignInRequested
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"

class SignInsTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val engine = SystemWebViewEngine(pages).also { it.attach(core) }

    /** What the engine was answered, in order: the web view, and the name and password, or null for no answer. */
    private val told = ArrayList<Pair<Long, Pair<String, String>?>>()

    private suspend fun aWebView(id: Long): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    /** A server asks who is there, as the page's web view would say it. */
    private fun ask(page: FakePage, host: String = "staff.example.com", realm: String = "Staff", again: Boolean = false) {
        page.host.signIn(page.id, host, realm, page.tracker.headingFor, again) { typed -> told.add(page.id to typed?.let { it.name to it.password }) }
    }

    private suspend fun answer(request: Long, accepted: Boolean, name: String = "", password: String = "") =
        engine.answer(HostRequest(sign_in_answer = SignInAnswer(request_id = request, accepted = accepted, name = name, password = password)))

    /** The questions the core was told of, in order. */
    private fun reported(): List<SignInRequested> = core.reports.mapNotNull { it.sign_in_requested }

    @Test
    fun aServerThatAsksIsReportedWithWhereTheLoadIsHeadingAndTheLoadWaits() =
        runTest {
            val page = aWebView(4)
            // The load was sent on from where it began: the policy let each hop go ahead.
            page.tracker.willLoad("https://example.com/start")
            page.tracker.navigationAllowed("https://staff.example.com/rota")

            ask(page, again = true)

            val asked = reported().single()
            assertEquals(4L, core.reports.last().web_view_id)
            assertEquals("staff.example.com", asked.host)
            assertEquals("the WebView names no origin", "", asked.origin)
            assertEquals("Staff", asked.realm)
            assertEquals("where the load is going now, not where it began", "https://staff.example.com/rota", asked.heading)
            assertTrue("the server refused before", asked.again)
            assertTrue("named by the shell", asked.request_id != 0L)
            assertTrue("the load waits for the core's word", told.isEmpty())
        }

    @Test
    fun withNoLoadOnItsWayNothingIsSaidOfWhereOneIsHeading() =
        runTest {
            val page = aWebView(4)
            ask(page)
            assertEquals("nothing was loading", "", reported().last().heading)

            // A page has arrived: what asks now is the page's own business.
            page.tracker.willLoad("https://example.com/")
            page.tracker.committed("https://example.com/", canGoBack = false, canGoForward = false)
            ask(page)
            assertEquals("", reported().last().heading)
            assertTrue("and it is not said to be asked again", !reported().last().again)
        }

    @Test
    fun theCoresWordIsTheServersAnswer() =
        runTest {
            val page = aWebView(4)
            ask(page)
            ask(page)
            val (first, second) = reported().map { it.request_id }
            assertTrue("each has a name of its own", first != second)

            assertNotNull(answer(second, accepted = false, name = "ada", password = "lovelace").ack)
            assertEquals("no answer, whatever came with it", listOf<Pair<Long, Pair<String, String>?>>(4L to null), told)

            assertNotNull(answer(first, accepted = true, name = "ada", password = "love lace").ack)
            assertEquals(4L to ("ada" to "love lace"), told.last())

            // Answered once: a second word is for nothing, and so is one about no question.
            assertNotNull(answer(first, accepted = false).ack)
            assertNotNull(answer(99, accepted = true, name = "x", password = "y").ack)
            assertEquals(2, told.size)
        }

    @Test
    fun aWebViewThatIsDestroyedGivesItsServersNoAnswerAndNotAnothers() =
        runTest {
            val going = aWebView(4)
            val staying = aWebView(5)
            ask(going)
            ask(staying)
            val kept = reported().last().request_id

            engine.answer(HostRequest(web_view_destroy = WebViewDestroy(web_view_id = 4)))

            assertEquals(listOf<Pair<Long, Pair<String, String>?>>(4L to null), told)
            assertTrue("the web view was closed after its servers were answered", going.closed)
            // The other still waits, and is answered by the core's word.
            answer(kept, accepted = true, name = "ada", password = "lovelace")
            assertEquals(5L to ("ada" to "lovelace"), told.last())
        }
}
