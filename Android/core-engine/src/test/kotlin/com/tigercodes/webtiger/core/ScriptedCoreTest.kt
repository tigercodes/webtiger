package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.CloseWindow
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.NavigationFinished
import com.tigercodes.webtiger.core.v1.OpenWindow
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.WindowOpened
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class ScriptedCoreTest {
    @Test
    fun requestsAreRecordedAndAnsweredByTheScript() =
        runTest {
            val core =
                ScriptedCore { request ->
                    if (request.open_window != null) Response(window_opened = WindowOpened(window_id = 7, tab_id = 8)) else Response(ack = Ack())
                }
            val opened = core.request(Request(open_window = OpenWindow())).window_opened
            assertEquals(7L, opened?.window_id)
            core.perform(Request(close_window = CloseWindow()))
            assertEquals(2, core.requests.size)
        }

    @Test
    fun aScriptCanRefuseByThrowingOrByAnsweringAnError() =
        runTest {
            val throwing = ScriptedCore { throw CoreError.Refused(code = "tab.unknown", field = "tab_id") }
            val answering = ScriptedCore { Response(error = Error(code = "tab.unknown", field_ = "tab_id")) }
            for (core in listOf(throwing, answering)) {
                try {
                    core.perform(Request(submit_address = SubmitAddress()))
                    fail("expected a refusal")
                } catch (refused: CoreError.Refused) {
                    assertEquals(CoreError.Refused(code = "tab.unknown", field = "tab_id"), refused)
                }
            }
        }

    @Test
    fun eventsArriveInOrderAndEndWhenStopped() =
        runTest {
            val core = ScriptedCore()
            core.emit(WindowSnapshot(window_id = 1), subscription = 5)
            core.emit(WindowSnapshot(window_id = 2), subscription = 5)
            core.stop()
            val seen = core.events.toList().map { it.snapshot?.window?.window_id }
            assertEquals(listOf(1L, 2L), seen)
            assertTrue(core.isStopped)
            try {
                core.perform(Request(open_window = OpenWindow()))
                fail("expected the core to have stopped")
            } catch (stopped: CoreError.Stopped) {
                assertEquals(CoreError.Stopped(), stopped)
            }
        }

    @Test
    fun verdictsDefaultToAllow() {
        val core = ScriptedCore()
        core.setVerdict(NavigationVerdict.CANCEL, "javascript:alert(1)")
        core.setVerdict(RequestVerdict.BLOCK, "https://tracker.example.com/pixel.gif")
        assertEquals(NavigationVerdict.ALLOW, core.navigationVerdict("https://example.com/", webViewId = 1, mainFrame = true, userGesture = false))
        assertEquals(NavigationVerdict.CANCEL, core.navigationVerdict("javascript:alert(1)", webViewId = 1, mainFrame = true, userGesture = false))
        assertEquals(RequestVerdict.ALLOW, core.requestVerdict("https://example.com/a.js", "https://example.com/", webViewId = 1, resource = ResourceType.SCRIPT))
        assertEquals(
            RequestVerdict.BLOCK,
            core.requestVerdict("https://tracker.example.com/pixel.gif", "https://example.com/", webViewId = 1, resource = ResourceType.IMAGE),
        )
    }

    @Test
    fun aFrameHasItsStorageUntilAScriptSaysOtherwise() {
        val core = ScriptedCore()
        val page = "https://news.example.com/story"
        // As an engine is by itself, until a test says what the core would.
        assertEquals(StorageVerdict.OWN, core.storageVerdict("https://tracker.example.net", "https://news.example.com", page, webViewId = 1))
        core.setStorage { frame -> if (frame.origin == "https://tracker.example.net") StorageVerdict.APART else StorageVerdict.OWN }
        assertEquals(StorageVerdict.APART, core.storageVerdict("https://tracker.example.net", "https://news.example.com", page, webViewId = 1))
        assertEquals(StorageVerdict.OWN, core.storageVerdict("https://comments.example.com", "https://news.example.com", page, webViewId = 2))
        // Every frame asked about is kept, in order, with all that was said of it.
        assertEquals(
            listOf(
                FrameAsked("https://tracker.example.net", "https://news.example.com", page, 1),
                FrameAsked("https://tracker.example.net", "https://news.example.com", page, 1),
                FrameAsked("https://comments.example.com", "https://news.example.com", page, 2),
            ),
            core.frames,
        )
    }

    @Test
    fun reportsAreRecorded() {
        val core = ScriptedCore()
        core.report(EngineEvent(web_view_id = 4, navigation_finished = NavigationFinished()))
        assertEquals(listOf(4L), core.reports.map { it.web_view_id })
    }
}
