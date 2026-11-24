// What a page is doing that would keep its tab awake (052), through the adapter, against the
// device's real WebView and the test site: that it was typed into, and that it plays a sound,
// each reported when it begins and ends, with all the adapter knows as it now is. What is
// typed here is typed as an editing command, which is what a key press is to the page; what is
// played is silence, started by a finger on the page's button, as the WebView requires. The
// Apple suite's WebKitActivityTests.swift, for Android.
package com.tigercodes.webtiger.webview

import android.Manifest
import androidx.test.platform.app.InstrumentationRegistry
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.PermissionAnswer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Test

internal class SystemWebViewActivityTest : AdapterTestCase() {
    /**
     * What the adapter has said the page of a web view is doing, in order: each as "sound",
     * "capture", "typed" joined by commas, and empty for none of them.
     */
    private fun doings(id: Long): List<String> =
        reports(id).mapNotNull { report ->
            report.page_activity_changed?.let { doing ->
                listOfNotNull("sound".takeIf { doing.plays_sound }, "capture".takeIf { doing.captures }, "typed".takeIf { doing.typed_into }).joinToString(",")
            }
        }

    private fun awaitDoings(id: Long, count: Int) {
        await(id, "$count reports of what the page is doing; said: ${doings(id)}") { doings(id).size >= count }
    }

    /** A web view on the page with the fields and the sound, in a window. */
    private fun page(id: Long = 1, path: String = "/doing", title: String = "Doing") {
        assertNotNull(create(id).ack)
        load(id, site.url(path))
        await(id, "the page") { finished(it) && hasTitle(title)(it) }
        show(id)
    }

    /** Types into a field of the page, as an editing command. */
    private fun type(text: String, into: String, id: Long = 1, inFrame: Boolean = false) {
        val document = if (inFrame) "frames[0].document" else "document"
        run(id, "$document.getElementById('$into').focus(); return $document.execCommand('insertText', false, '$text');")
    }

    /** Nothing more is said in a while: what was said is all there is. */
    private fun nothingMore(id: Long, than: Int) {
        Thread.sleep(600)
        assertEquals("said: ${doings(id)}", than, doings(id).size)
    }

    // Typing

    @Test
    fun typingIntoAFieldIsSaidOnceAndNothingOfWhatWasTyped() {
        page()
        nothingMore(1, 0)

        type("a draft", "line")
        awaitDoings(1, 1)
        assertEquals(listOf("typed"), doings(1))
        // More of it changes nothing: it is said when it begins.
        type(" of a letter", "line")
        type("and more", "lines")
        nothingMore(1, 1)
        // Nothing of what was typed is in anything the adapter reported.
        val said = core.reports.joinToString { it.toString() }
        assertFalse(said.contains("draft") || said.contains("letter"))
    }

    @Test
    fun aLargerFieldAndWordsThatCanBeEditedAreTypedIntoAndABoxThatIsTickedIsNot() {
        page(1)
        type("lines", "lines", 1)
        awaitDoings(1, 1)
        assertEquals(listOf("typed"), doings(1))

        page(2)
        type("more ", "words", 2)
        awaitDoings(2, 1)
        assertEquals(listOf("typed"), doings(2))

        // A box is ticked, by a finger: that is no typing.
        page(3)
        tap(3, "tick")
        await(3, "the box to take the finger") { run(3, "return document.getElementById('tick').checked;") == "true" }
        nothingMore(3, 0)
    }

    @Test
    fun whatAPagesOwnScriptPutsInAFieldIsNotTypingAndThePageCannotReachTheAdaptersEar() {
        page()
        // The page fills its field itself, and says an input happened: no key went in.
        run(1, "const line = document.getElementById('line'); line.value = 'filled in'; line.dispatchEvent(new InputEvent('input', { bubbles: true }));")
        nothingMore(1, 0)
        // The ear's name is gone from the page before any script of the page's ran.
        assertEquals("\"undefined\"", run(1, "return typeof ${PageDoing.NAME};"))
    }

    @Test
    fun aFrameThatIsTypedIntoIsThePageTypedInto() {
        page(1, "/doing-in-a-frame", "Doing in a frame")
        await(1, "the frame's page") { run(1, "return !!(frames[0] && frames[0].document.getElementById('line'));") == "true" }
        type("in the frame", "line", 1, inFrame = true)
        awaitDoings(1, 1)
        assertEquals(listOf("typed"), doings(1))
    }

    @Test
    fun whatWasTypedIsOfThePageAndAPageThatFollowsStartsWithNone() {
        page()
        type("a draft", "line")
        awaitDoings(1, 1)

        // Another page commits. The adapter says nothing of it by itself: the core takes a
        // commit as the end of what the page before was doing.
        load(1, site.url("/tall"))
        await(1, "the next page") { finished(it, 2) && hasTitle("Tall")(it) }
        nothingMore(1, 1)
        // Typed into in its turn, the new page is said to be: the adapter began again.
        type("another", "note")
        awaitDoings(1, 2)
        assertEquals(listOf("typed", "typed"), doings(1))
        // In the order of things: the second was said after the second page's commit.
        val order = reports(1).mapNotNull { report ->
            when {
                report.navigation_committed != null -> "committed"
                report.page_activity_changed != null -> "doing"
                else -> null
            }
        }
        assertEquals(listOf("committed", "doing", "committed", "doing"), order)
    }

    // Sound

    @Test
    fun aSoundThatPlaysIsSaidAndSaidToHaveStopped() {
        page()
        tap(1, "play")
        awaitDoings(1, 1)
        assertEquals(listOf("sound"), doings(1))
        // It plays on, again and again: nothing more is said while it does.
        nothingMore(1, 1)

        tap(1, "stop")
        awaitDoings(1, 2)
        assertEquals(listOf("sound", ""), doings(1))

        // Typed into, and playing again: all the adapter knows, as it now is.
        type("a draft", "line")
        awaitDoings(1, 3)
        tap(1, "play")
        awaitDoings(1, 4)
        assertEquals(listOf("sound", "", "typed", "sound,typed"), doings(1))
    }

    @Test
    fun aSoundThatIsTurnedOffOrAllTheWayDownIsNotASound() {
        page()
        // Turned off before it plays: it plays, and nothing is heard.
        run(1, "document.getElementById('tune').muted = true;")
        tap(1, "play")
        await(1, "the sound to be playing") { run(1, "return !document.getElementById('tune').paused;") == "true" }
        nothingMore(1, 0)
        // Turned on again while it plays: now it is a sound.
        run(1, "document.getElementById('tune').muted = false;")
        awaitDoings(1, 1)
        assertEquals(listOf("sound"), doings(1))
        // All the way down: not a sound any more; and up again.
        run(1, "document.getElementById('tune').volume = 0;")
        awaitDoings(1, 2)
        run(1, "document.getElementById('tune').volume = 0.5;")
        awaitDoings(1, 3)
        assertEquals(listOf("sound", "", "sound"), doings(1))
    }

    @Test
    fun aSoundGoesWithItsPage() {
        page()
        tap(1, "play")
        awaitDoings(1, 1)
        load(1, site.url("/landed"))
        await(1, "the next page") { finished(it, 2) && hasTitle("Landed")(it) }
        Thread.sleep(400)
        // Nothing is said of a sound after the commit of the page that plays none.
        val reported = reports(1)
        val second = reported.indices.filter { reported[it].navigation_committed != null }[1]
        assertFalse(reported.drop(second).any { it.page_activity_changed?.plays_sound == true })
    }

    // The camera and the microphone

    @Test
    fun aPageThatIsGivenTheCameraIsSaidToHoldItUntilItGoes() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        instrumentation.uiAutomation.grantRuntimePermission(instrumentation.targetContext.packageName, Manifest.permission.CAMERA)
        assertNotNull(create(1).ack)
        load(1, site.url("/asks"))
        await(1, "the page") { finished(it) && hasTitle("Asks")(it) }
        tap(1, "camera")
        await(1, "the page to ask") { reports -> reports.any { it.permission_requested != null } }
        nothingMore(1, 0)
        // The core says yes: the page has the camera, and the adapter says it holds one.
        val asked = checkNotNull(reports(1).last { it.permission_requested != null }.permission_requested)
        assertNotNull(answer(HostRequest(permission_answer = PermissionAnswer(request_id = asked.request_id, granted = true))).ack)
        awaitDoings(1, 1)
        assertEquals(listOf("capture"), doings(1))
        // A page that is refused holds nothing: another web view, told no.
        assertNotNull(create(2).ack)
        load(2, site.url("/asks"))
        await(2, "the page") { finished(it) && hasTitle("Asks")(it) }
        tap(2, "camera")
        await(2, "the page to ask") { reports -> reports.any { it.permission_requested != null } }
        val refused = checkNotNull(reports(2).last { it.permission_requested != null }.permission_requested)
        assertNotNull(answer(HostRequest(permission_answer = PermissionAnswer(request_id = refused.request_id, granted = false))).ack)
        nothingMore(2, 0)
    }
}
