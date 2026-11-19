// The shell's own host (007): the System service is answered here, by an opener a test replaces
// with one that only writes down what it was asked; everything else is the engine's. And the
// system's own opener: the intent it starts.
package com.tigercodes.webtiger.ui

import android.app.Application
import android.content.Intent
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.HostServices
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.HostPing
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.OpenOutside
import com.tigercodes.webtiger.core.v1.WebViewCommand
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import com.tigercodes.webtiger.core.v1.WebViewLoad
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf

@RunWith(RobolectricTestRunner::class)
class ShellHostTest {
    /** An engine that writes down what it was asked and acknowledges it. */
    private class Engine : HostServices {
        val asked = ArrayList<HostRequest>()

        override suspend fun answer(request: HostRequest): HostResponse {
            asked.add(request)
            return HostResponse(ack = Ack())
        }
    }

    private val engine = Engine()
    private val opened = ArrayList<String>()

    private fun host(opens: Boolean = true) =
        ShellHost(engine) { url ->
            opened.add(url)
            opens
        }

    private fun openOutside(url: String) = HostRequest(open_outside = OpenOutside(url = url))

    @Test
    fun anOutsideLinkGoesToTheOpenerAsItWasGiven() =
        runTest {
            val answer = host().answer(openOutside("mailto:ada@example.com?subject=Hello"))
            assertNotNull("the system took it", answer.ack)
            assertEquals(listOf("mailto:ada@example.com?subject=Hello"), opened)
            assertTrue("an engine is asked nothing about it", engine.asked.isEmpty())
        }

    @Test
    fun nothingOnTheDeviceOpensItIsACode() =
        runTest {
            val answer = host(opens = false).answer(openOutside("tel:+15550100"))
            assertEquals("system.no_handler", answer.error?.code)
            assertNull(answer.ack)
            assertEquals("the system was asked", 1, opened.size)
        }

    @Test
    fun everyOtherRequestIsTheEngines() =
        runTest {
            val requests =
                listOf(
                    HostRequest(ping = HostPing()),
                    HostRequest(web_view_create = WebViewCreate(web_view_id = 4)),
                    HostRequest(web_view_load = WebViewLoad(web_view_id = 4)),
                    HostRequest(web_view_command = WebViewCommand(web_view_id = 4)),
                    HostRequest(web_view_destroy = WebViewDestroy(web_view_id = 4)),
                    // And one this shell does not know: the engine says so.
                    HostRequest(),
                )
            val host = host()
            for (request in requests) assertNotNull("the engine's answer comes back", host.answer(request).ack)
            assertEquals(requests, engine.asked)
            assertTrue(opened.isEmpty())
        }

    @Test
    fun fakeModesOpenerOpensNothingAndSaysItDid() =
        runTest {
            val application = ApplicationProvider.getApplicationContext<Application>()
            val answer = ShellHost(engine, OutsideOpener.nothing).answer(openOutside("mailto:ada@example.com"))
            assertNotNull("fake mode answers as if the system had taken it", answer.ack)
            assertNull("and no activity was started", shadowOf(application).nextStartedActivity)
        }

    // The system's own opener

    @Test
    fun theSystemsOpenerStartsAnActivityThatViewsTheAddress() {
        val application = ApplicationProvider.getApplicationContext<Application>()

        assertTrue(SystemOpener(application).open("mailto:ada@example.com?subject=Hello"))

        val started = shadowOf(application).nextStartedActivity
        assertEquals(Intent.ACTION_VIEW, started.action)
        assertEquals("mailto:ada@example.com?subject=Hello", started.dataString)
        assertTrue("only an activity that offers itself to links from the web", started.hasCategory(Intent.CATEGORY_BROWSABLE))
        assertTrue("in a task of its own, not in the browser's", started.flags and Intent.FLAG_ACTIVITY_NEW_TASK != 0)
        assertNull("the system chooses what opens it; the page names no app", started.component)
        assertNull(started.`package`)
        assertNull("and only this one was started", shadowOf(application).nextStartedActivity)
    }

    @Test
    fun theSystemsOpenerSaysWhenNothingOpensAnAddress() {
        val application = ApplicationProvider.getApplicationContext<Application>()
        // As a device does: an intent no activity takes is refused, not dropped.
        shadowOf(application).checkActivities(true)

        assertFalse(SystemOpener(application).open("tel:+15550100"))
    }

    @Test
    fun whatIsNotAnAddressIsNotGivenToTheSystem() {
        val application = ApplicationProvider.getApplicationContext<Application>()
        for (text in listOf("", "not an address", "ada@example.com")) assertFalse(text, SystemOpener(application).open(text))
        assertNull(shadowOf(application).nextStartedActivity)
    }
}
