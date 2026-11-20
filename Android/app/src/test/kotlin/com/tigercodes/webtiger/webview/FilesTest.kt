// A page that asks for a file (019), with no WebView behind the adapter: what the chooser is
// asked, what the page is given, and when it is given nothing. Through the engine, with a
// chooser of the test's own that answers when the test says.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val NOTES = "content://files/notes.txt"
private const val PHOTO = "content://files/photo.jpg"

class FilesTest {
    /** What the chooser was asked, in order, and the answers it has yet to give. */
    private val asked = ArrayList<FilesAsked>()
    private val open = ArrayList<(List<String>) -> Unit>()
    private val chooser =
        FileChooser { wanted, done ->
            asked.add(wanted)
            open.add(done)
        }
    private val pages = FakePages()
    private val engine = SystemWebViewEngine(pages, chooser = chooser).also { it.attach(ScriptedCore()) }

    /** What each page was given, in order: the web view, and the files' addresses or null for nothing. */
    private val given = ArrayList<Pair<Long, List<String>?>>()

    private suspend fun aWebView(id: Long, engine: SystemWebViewEngine = this.engine): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    /** A page's file input is pressed, as its web view would say it. */
    private fun ask(page: FakePage, several: Boolean = false, kinds: List<String> = emptyList()) {
        page.host.files(page.id, FilesAsked(several, kinds)) { files -> given.add(page.id to files) }
    }

    @Test
    fun aPageThatAsksIsGivenWhatWasPickedAndTheChooserIsToldWhatItMayHave() =
        runTest {
            val page = aWebView(4)

            ask(page, several = true, kinds = listOf("image/*", ".txt"))

            assertEquals(1, asked.size)
            assertTrue("several may be chosen", asked.single().several)
            assertEquals("the kinds the page says it accepts", listOf("image/*", ".txt"), asked.single().kinds)
            assertTrue("the page waits for the person", given.isEmpty())

            open.single()(listOf(NOTES, PHOTO))

            assertEquals(listOf<Pair<Long, List<String>?>>(4L to listOf(NOTES, PHOTO)), given)
        }

    @Test
    fun aPageThatMayHaveOneFileIsGivenNoMore() =
        runTest {
            val page = aWebView(4)

            ask(page, several = false)
            open.single()(listOf(NOTES, PHOTO))

            assertEquals(listOf<Pair<Long, List<String>?>>(4L to listOf(NOTES)), given)
        }

    @Test
    fun nothingPickedIsNothingAndThePageMayAskAgain() =
        runTest {
            val page = aWebView(4)

            ask(page)
            open[0](emptyList())
            assertEquals(listOf<Pair<Long, List<String>?>>(4L to null), given)

            ask(page)
            assertEquals("the chooser is opened again", 2, asked.size)
            open[1](listOf(NOTES))
            assertEquals(4L to listOf(NOTES), given.last())
        }

    @Test
    fun oneChooserAtATimeForAWebView() =
        runTest {
            val page = aWebView(4)
            val other = aWebView(5)

            ask(page)
            ask(page)

            // The second is told nothing, at once, and no second chooser is opened for it.
            assertEquals(listOf<Pair<Long, List<String>?>>(4L to null), given)
            assertEquals(1, asked.size)
            // Another web view's page has a chooser of its own.
            ask(other)
            assertEquals(2, asked.size)
            // An answer is for the request that opened the chooser, and is given once.
            open[0](listOf(NOTES))
            open[0](listOf(PHOTO))
            assertEquals(listOf(4L to null, 4L to listOf(NOTES)), given)
        }

    @Test
    fun aWebViewThatIsDestroyedIsToldNothingAndWhatIsThenPickedGoesNowhere() =
        runTest {
            val going = aWebView(4)
            val staying = aWebView(5)
            ask(going)
            ask(staying)

            engine.answer(HostRequest(web_view_destroy = WebViewDestroy(web_view_id = 4)))

            assertEquals(listOf<Pair<Long, List<String>?>>(4L to null), given)
            // The person picks all the same: it goes nowhere, and the other page has its own.
            open[0](listOf(NOTES))
            open[1](listOf(PHOTO))
            assertEquals(listOf(4L to null, 5L to listOf(PHOTO)), given)
        }

    @Test
    fun anEngineGivenNoChooserPicksNothing() =
        runTest {
            val plain = SystemWebViewEngine(pages).also { it.attach(ScriptedCore()) }
            val page = aWebView(4, plain)

            ask(page, several = true)

            assertEquals(listOf<Pair<Long, List<String>?>>(4L to null), given)
            assertTrue("and the test's chooser was not the one asked", asked.isEmpty())
        }

    @Test
    fun theKindsAPageAcceptsAreGivenToTheSystemsPickerAsTypes() {
        val known = mapOf("txt" to "text/plain", "jpg" to "image/jpeg")
        val typeOf: (String) -> String? = { ending -> known[ending] }

        // A type is itself; an ending is the type the system knows it by; each once.
        assertEquals(listOf("image/*", "text/plain", "image/jpeg"), pickerTypes(listOf("image/*", ".TXT", " .jpg ", "text/plain"), typeOf))
        // What says nothing the picker can go by is left out; with nothing left, every file is offered.
        assertEquals(listOf("*/*"), pickerTypes(listOf(".unheard-of", "nonsense"), typeOf))
        assertEquals(listOf("*/*"), pickerTypes(emptyList(), typeOf))
    }

    @Test
    fun theAppsChooserPicksNothingUntilAWindowGivesItTheWayToTheSystems() {
        val files = AppFiles()
        val picked = ArrayList<List<String>>()

        files.choose(FilesAsked(several = false)) { picked.add(it) }
        assertEquals("with no window, nothing", listOf(emptyList<String>()), picked)

        val started = ArrayList<FilesAsked>()
        files.picker = { wanted, done ->
            started.add(wanted)
            done(listOf(NOTES))
        }
        files.choose(FilesAsked(several = true, kinds = listOf("text/plain"))) { picked.add(it) }
        assertEquals(listOf(NOTES), picked.last())
        assertTrue(started.single().several)
        assertEquals(listOf("text/plain"), started.single().kinds)
    }
}
