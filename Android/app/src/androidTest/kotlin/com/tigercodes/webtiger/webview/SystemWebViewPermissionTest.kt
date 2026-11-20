// The adapter suite's System WebView run, for what a page asks to use of the device's (014):
// the camera, the microphone, where the device is. What the adapter reports, that the page
// waits for the core's word, and what each word does to the page. The system's own dialog is
// never brought up: the engine uses what the app holds, and what a test needs the app to hold
// it is given here, without a dialog.
package com.tigercodes.webtiger.webview

import android.Manifest
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.PermissionAnswer
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_CAMERA
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_LOCATION
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_MICROPHONE
import com.tigercodes.webtiger.core.v1.PermissionRequested
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewPermissionTest : AdapterTestCase() {
    /** The app's own permissions, which a test can have say that the system refuses the app. */
    private class Switchable(private val own: SystemPermissions) : SystemPermissions {
        var systemRefuses = false
        var asked = 0

        override fun holds(kinds: List<PermissionKind>): Boolean = !systemRefuses && own.holds(kinds)

        override fun ask(kinds: List<PermissionKind>, done: (Boolean) -> Unit) {
            asked += 1
            if (systemRefuses) done(false) else own.ask(kinds, done)
        }
    }

    private lateinit var system: Switchable

    override fun systemPermissions(context: Context): SystemPermissions = Switchable(AppPermissions(context)).also { system = it }

    /** Gives the app a permission of the system's, as the person would by its dialog, with no dialog. */
    private fun theAppHolds(vararg permissions: String) {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        for (permission in permissions) instrumentation.uiAutomation.grantRuntimePermission(instrumentation.targetContext.packageName, permission)
    }

    /** A web view on the page that asks by its buttons. */
    private fun page(id: Long = 1) {
        create(id)
        load(id, site.url("/asks"))
        await(id, "the page to load") { finished(it) && hasTitle("Asks")(it) }
    }

    /** What the adapter has reported a web view's pages asking, in order. */
    private fun asked(id: Long = 1): List<PermissionRequested> = reports(id).mapNotNull { it.permission_requested }

    /** Taps one of the page's buttons with a finger, and waits for what it asks to be reported. */
    private fun ask(what: String, id: Long = 1): PermissionRequested {
        val before = asked(id).size
        tap(id, what)
        await(id, "the page to ask") { asked(id).size > before }
        return asked(id).last()
    }

    /** The core's word on a request. */
    private fun say(request: PermissionRequested, granted: Boolean) {
        assertNotNull(answer(HostRequest(permission_answer = PermissionAnswer(request_id = request.request_id, granted = granted))).ack)
    }

    /** Waits for the page to say, in its title, what became of what it asked. */
    private fun awaitSaid(words: String, id: Long = 1) {
        await(id, "the page to say \"$words\"; it last said \"${lastTitle(id)}\"") { hasTitle(words)(it) }
    }

    /** The page's own origin, as an origin is written. */
    private val origin: String get() = site.url("/").trimEnd('/')

    @Test
    fun aPageThatAsksForTheCameraIsReportedAndWaitsAndIsToldNo() {
        page()

        val request = ask("camera")

        assertEquals(listOf(PERMISSION_KIND_CAMERA), request.kinds)
        assertEquals("who asks: the page's origin", origin, request.origin.trimEnd('/'))
        assertNotEquals("named by the shell", 0L, request.request_id)
        // The page waits: it has said it is asking, and nothing more.
        Thread.sleep(500)
        assertEquals("asking camera", lastTitle(1))

        say(request, false)

        awaitSaid("refused camera NotAllowedError")
        assertEquals("the system was asked nothing", 0, system.asked)
    }

    @Test
    fun theMicrophoneAndBothTogetherAreReportedAsWhatTheyAre() {
        page()

        val microphone = ask("microphone")
        assertEquals(listOf(PERMISSION_KIND_MICROPHONE), microphone.kinds)
        say(microphone, false)
        awaitSaid("refused microphone NotAllowedError")

        val both = ask("both")
        assertEquals(setOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE), both.kinds.toSet())
        assertEquals(2, both.kinds.size)
        assertNotEquals("each request has a name of its own", microphone.request_id, both.request_id)
        say(both, false)
        awaitSaid("refused both NotAllowedError")
    }

    @Test
    fun aRequestTheCoreGrantsGivesThePageTheCameraWhenTheAppHoldsIt() {
        theAppHolds(Manifest.permission.CAMERA)
        page()

        val request = ask("camera")
        say(request, true)

        // The emulator's camera, through the WebView: one track of video.
        awaitSaid("granted camera 1")
        assertEquals("the app held it: the system was not asked", 0, system.asked)
    }

    @Test
    fun cameraAndMicrophoneAreGrantedTogether() {
        theAppHolds(Manifest.permission.CAMERA, Manifest.permission.RECORD_AUDIO)
        page()

        say(ask("both"), true)

        awaitSaid("granted both 2")
    }

    @Test
    fun aSiteTheCoreAllowsIsRefusedWhenTheSystemRefusesTheApp() {
        page()
        system.systemRefuses = true

        val request = ask("camera")
        say(request, true)

        awaitSaid("refused camera NotAllowedError")
        assertEquals("the system was asked, and said no", 1, system.asked)
    }

    @Test
    fun whereTheDeviceIsIsAskedForAndRefusedAndGranted() {
        page()

        val request = ask("location")

        assertEquals(listOf(PERMISSION_KIND_LOCATION), request.kinds)
        assertEquals(origin, request.origin.trimEnd('/'))
        Thread.sleep(500)
        assertEquals("the page waits", "asking location", lastTitle(1))
        say(request, false)
        awaitSaid("refused location")

        // Granted, with the system's permission given to the test: the page is not refused.
        // Whether the emulator then knows where it is, is the emulator's affair.
        theAppHolds(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION)
        val again = ask("location")
        assertNotEquals(request.request_id, again.request_id)
        say(again, true)
        await(1, "the page to say it was granted; it last said \"${lastTitle(1)}\"") { reports -> reports.any { it.title_changed?.title.orEmpty().startsWith("granted location") } }
    }

    @Test
    fun aPageAsksAgainEachTimeAndIsAnsweredEachTime() {
        page()

        // The WebView keeps no answer of its own: what a site was allowed is the core's to
        // remember, and every request is the core's to answer.
        say(ask("camera"), false)
        awaitSaid("refused camera NotAllowedError")
        val second = ask("camera")
        say(second, false)

        assertEquals(2, asked().size)
    }

    @Test
    fun aFrameFromAnotherOriginIsReportedAsItself() {
        create(1)
        load(1, site.url("/asks-in-a-frame?now=camera"))

        await(1, "the frame to ask") { asked().isNotEmpty() }

        // The frame is the same server by its other name: another origin than the page's.
        // The adapter says who asks; whether a frame may is the core's rule (014 §3).
        val request = asked().last()
        assertEquals("http://localhost:${site.port}", request.origin.trimEnd('/'))
        assertEquals(listOf(PERMISSION_KIND_CAMERA), request.kinds)
        say(request, false)
        awaitSaid("frame: refused camera NotAllowedError")
    }

    @Test
    fun aWebViewDestroyedWhileItsPageAsksTakesTheRequestWithIt() {
        page()
        val request = ask("camera")

        destroy(1)

        // A word for it afterwards is for nothing, and is acknowledged.
        say(request, true)
        assertTrue(asked().size == 1)
    }

    @Test
    fun aWordAboutARequestThereIsNot() {
        assertNotNull(answer(HostRequest(permission_answer = PermissionAnswer(request_id = 99, granted = true))).ack)
    }
}
