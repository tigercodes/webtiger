// A page asks to use something of the device's (014), with no WebView and no system behind
// the adapter: what is reported, that nothing is answered until the core says, and what the
// system's own permission has to do with the answer. Through the engine, as the core speaks
// to it.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.PermissionAnswer
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_CAMERA
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_LOCATION
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_MICROPHONE
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val MEET = "https://meet.example.com"

/** A system that says what a test tells it to, and writes down what it was asked for. */
private class FakeSystem : SystemPermissions {
    val held = HashSet<PermissionKind>()

    /** Every time the system was asked, in order: for what. */
    val asked = ArrayList<List<PermissionKind>>()

    /** What it says when asked. Null: it says nothing until the test lets it ([answer]). */
    var says: Boolean? = false
    private val unanswered = ArrayList<Pair<List<PermissionKind>, (Boolean) -> Unit>>()

    override fun holds(kinds: List<PermissionKind>): Boolean = held.containsAll(kinds)

    override fun ask(kinds: List<PermissionKind>, done: (Boolean) -> Unit) {
        asked.add(kinds)
        val says = says
        if (says == null) {
            unanswered.add(kinds to done)
        } else {
            if (says) held.addAll(kinds)
            done(says)
        }
    }

    /** The person answers the system's dialog that has been up longest. */
    fun answer(yes: Boolean) {
        val (kinds, done) = unanswered.removeAt(0)
        if (yes) held.addAll(kinds)
        done(yes)
    }
}

class PermissionsTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val system = FakeSystem()
    private val engine = SystemWebViewEngine(pages, system = system).also { it.attach(core) }

    /** What each page was told, in order: the web view, and yes or no. */
    private val told = ArrayList<Pair<Long, Boolean>>()

    private suspend fun aWebView(id: Long): FakePage {
        engine.answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = EngineKind.ENGINE_KIND_CHROMIUM, login_set = LoginSet(id = LOGIN_SET))))
        return pages.made.last()
    }

    /** A page asks, as its web view would say it. The adapter's name for the request. */
    private fun ask(page: FakePage, vararg kinds: PermissionKind, origin: String = MEET): Long =
        page.host.permissionAsked(page.id, origin, kinds.toList()) { granted -> told.add(page.id to granted) }

    private suspend fun say(request: Long, granted: Boolean) = engine.answer(HostRequest(permission_answer = PermissionAnswer(request_id = request, granted = granted)))

    /** What the core was told of requests, each in a few words. */
    private fun reported(): List<String> =
        core.reports.mapNotNull { report ->
            val asked = report.permission_requested
            val withdrawn = report.permission_withdrawn
            when {
                asked != null -> "asked ${asked.request_id} by ${report.web_view_id} as ${asked.origin}: ${asked.kinds.joinToString { it.name.removePrefix("PERMISSION_KIND_").lowercase() }}"
                withdrawn != null -> "withdrawn ${withdrawn.request_id} by ${report.web_view_id}"
                else -> null
            }
        }

    @Test
    fun aPageThatAsksIsReportedWithWhoAsksAndForWhatAndWaits() =
        runTest {
            val page = aWebView(4)

            val request = ask(page, PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)

            assertEquals(listOf("asked $request by 4 as $MEET: camera, microphone"), reported())
            assertTrue("the page waits for the core's word", told.isEmpty())
            assertTrue("and the system is asked nothing on a page's word", system.asked.isEmpty())
        }

    @Test
    fun eachRequestHasANameOfTheShellsOwn() =
        runTest {
            val first = aWebView(4)
            val second = aWebView(5)

            val names = listOf(ask(first, PERMISSION_KIND_CAMERA), ask(second, PERMISSION_KIND_CAMERA), ask(first, PERMISSION_KIND_LOCATION))

            assertEquals("each its own, and none used twice", 3, names.toSet().size)
            assertTrue(names.none { it == 0L })
            say(names[0], false)
            val next = ask(first, PERMISSION_KIND_CAMERA)
            assertTrue("a name that was answered is not given again", next !in names)
        }

    @Test
    fun aRequestTheCoreRefusesIsRefusedAndTheSystemIsNotAsked() =
        runTest {
            val page = aWebView(4)
            val request = ask(page, PERMISSION_KIND_CAMERA)

            assertNotNull(say(request, false).ack)

            assertEquals(listOf(4L to false), told)
            assertTrue(system.asked.isEmpty())
        }

    @Test
    fun aRequestTheCoreGrantsIsGrantedWhenTheAppHoldsWhatTheSystemWants() =
        runTest {
            val page = aWebView(4)
            system.held.add(PERMISSION_KIND_CAMERA)
            val request = ask(page, PERMISSION_KIND_CAMERA)

            assertNotNull(say(request, true).ack)

            assertEquals(listOf(4L to true), told)
            assertTrue("it holds it: the system is not asked again", system.asked.isEmpty())
        }

    @Test
    fun whatTheAppDoesNotHoldTheSystemIsAskedForAndItsAnswerIsThePages() =
        runTest {
            val page = aWebView(4)
            system.says = null
            val camera = ask(page, PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)
            val place = ask(page, PERMISSION_KIND_LOCATION)

            say(camera, true)
            say(place, true)

            assertEquals("asked for what each request needs", listOf(listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE), listOf(PERMISSION_KIND_LOCATION)), system.asked)
            assertTrue("the pages wait for the system", told.isEmpty())

            // The person lets the app have the camera and the microphone, and not the location.
            system.answer(true)
            system.answer(false)

            assertEquals("the site was allowed; the device was not", listOf(4L to true, 4L to false), told)
        }

    @Test
    fun aRequestThePageWithdrawsIsReportedAndNeverAnswered() =
        runTest {
            val page = aWebView(4)
            val request = ask(page, PERMISSION_KIND_CAMERA)

            page.host.permissionWithdrawn(request)

            assertEquals("withdrawn $request by 4", reported().last())
            // The core's word, on its way meanwhile, is for nothing.
            assertNotNull(say(request, true).ack)
            assertTrue(told.isEmpty())
            assertTrue(system.asked.isEmpty())
            // Withdrawn twice, and a name there never was: nothing is said.
            val before = reported().size
            page.host.permissionWithdrawn(request)
            page.host.permissionWithdrawn(999)
            assertEquals(before, reported().size)
        }

    @Test
    fun aWebViewThatIsDestroyedRefusesWhatItsPagesAsked() =
        runTest {
            val going = aWebView(4)
            val staying = aWebView(5)
            val gone = ask(going, PERMISSION_KIND_CAMERA)
            ask(going, PERMISSION_KIND_LOCATION)
            val kept = ask(staying, PERMISSION_KIND_CAMERA)

            engine.answer(HostRequest(web_view_destroy = WebViewDestroy(web_view_id = 4)))

            assertEquals("both of its own, and not another web view's", listOf(4L to false, 4L to false), told)
            assertTrue("the page was told before it was closed", going.closed)
            // A word for one of them afterwards is for nothing; the other still waits.
            say(gone, true)
            assertEquals(2, told.size)
            say(kept, false)
            assertEquals(5L to false, told.last())
        }

    @Test
    fun aWordAboutARequestThereIsNot() =
        runTest {
            aWebView(4)

            assertNotNull("done, as for one whose page went away", say(99, true).ack)

            assertTrue(told.isEmpty())
            assertTrue(system.asked.isEmpty())
        }

    @Test
    fun aRequestIsAnsweredOnce() =
        runTest {
            val page = aWebView(4)
            val request = ask(page, PERMISSION_KIND_CAMERA)

            say(request, false)
            say(request, true)

            assertEquals(listOf(4L to false), told)
        }
}

/** The app's own permissions (014): what it holds, and the one question at a time it puts to the system. */
class AppPermissionsTest {
    private val held = HashSet<String>()

    /** Every question put to the system, in order: the permissions' names. */
    private val asked = ArrayList<List<String>>()
    private val unanswered = ArrayList<() -> Unit>()
    private val permissions = AppPermissions { name -> name in held }

    /** What each question came to, in order. */
    private val answers = ArrayList<Boolean>()

    private fun aWindowToAskFrom() {
        permissions.asker = { names, done ->
            asked.add(names.toList())
            unanswered.add(done)
        }
    }

    /** The person answers the system's dialog: it grants these. */
    private fun theSystemGrants(vararg names: String) {
        held.addAll(names)
        unanswered.removeAt(0)()
    }

    private fun ask(vararg kinds: PermissionKind) = permissions.ask(kinds.toList()) { answers.add(it) }

    @Test
    fun whatTheAppHoldsIsWhatTheSystemGranted() {
        assertTrue(!permissions.holds(listOf(PERMISSION_KIND_CAMERA)))
        held.add("android.permission.CAMERA")
        assertTrue(permissions.holds(listOf(PERMISSION_KIND_CAMERA)))
        assertTrue("both, or it does not hold them", !permissions.holds(listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)))
        held.add("android.permission.RECORD_AUDIO")
        assertTrue(permissions.holds(listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)))
        // Where the device roughly is, is where it is: a person may give no more.
        assertTrue(!permissions.holds(listOf(PERMISSION_KIND_LOCATION)))
        held.add("android.permission.ACCESS_COARSE_LOCATION")
        assertTrue(permissions.holds(listOf(PERMISSION_KIND_LOCATION)))
        // What is no kind needs something nobody can hold.
        assertTrue(!permissions.holds(listOf(PermissionKind.PERMISSION_KIND_UNSPECIFIED)))
    }

    @Test
    fun withNoWindowToAskFromNobodyIsAsked() {
        held.add("android.permission.CAMERA")

        ask(PERMISSION_KIND_CAMERA)
        ask(PERMISSION_KIND_MICROPHONE)

        assertEquals("what the app holds, and no more", listOf(true, false), answers)
        assertTrue(asked.isEmpty())
    }

    @Test
    fun theSystemIsAskedForWhatAKindNeedsAndTheAnswerIsWhatTheAppThenHolds() {
        aWindowToAskFrom()

        ask(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)

        assertEquals(listOf(listOf("android.permission.CAMERA", "android.permission.RECORD_AUDIO")), asked)
        assertTrue("nothing is said until the system has answered", answers.isEmpty())
        // The person grants one of the two: the request needed both.
        theSystemGrants("android.permission.CAMERA")
        assertEquals(listOf(false), answers)

        // The location is asked for both ways, and held when either is granted.
        ask(PERMISSION_KIND_LOCATION)
        assertEquals(listOf("android.permission.ACCESS_FINE_LOCATION", "android.permission.ACCESS_COARSE_LOCATION"), asked.last())
        theSystemGrants("android.permission.ACCESS_COARSE_LOCATION")
        assertEquals(listOf(false, true), answers)
    }

    @Test
    fun theSystemIsAskedOneThingAtATimeAndNotAgainForWhatItHasGranted() {
        aWindowToAskFrom()

        ask(PERMISSION_KIND_CAMERA)
        ask(PERMISSION_KIND_CAMERA)
        ask(PERMISSION_KIND_LOCATION)

        assertEquals("one question is up; the rest wait", 1, asked.size)
        theSystemGrants("android.permission.CAMERA")

        assertEquals("the second wanted the same, and has it unasked", listOf(true, true), answers)
        assertEquals("the third is put to the system now", listOf(listOf("android.permission.CAMERA"), listOf("android.permission.ACCESS_FINE_LOCATION", "android.permission.ACCESS_COARSE_LOCATION")), asked)
        theSystemGrants()
        assertEquals(listOf(true, true, false), answers)
        // What the app holds is not asked for at all.
        ask(PERMISSION_KIND_CAMERA)
        assertEquals(2, asked.size)
        assertEquals(true, answers.last())
    }
}
