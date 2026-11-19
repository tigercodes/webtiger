// The zoom bar (012), drawn by Compose on Robolectric's Android over a scripted core: when it
// is there, what it says, and what its buttons send.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.ZoomChange
import com.tigercodes.webtiger.core.v1.ZoomView
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ZoomBarTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun show() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
    }

    /** The core says the page in front is at `percent`. */
    private fun aPageAt(percent: Int, canZoomIn: Boolean = true, canZoomOut: Boolean = true) {
        compose.runOnIdle {
            core.emit(
                snapshot(url = "https://example.com/", display = "example.com", webView = 4) {
                    it.copy(zoom = ZoomView(percent = percent, can_zoom_in = canZoomIn, can_zoom_out = canZoomOut))
                },
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    /** What a screen reader is told the level is. */
    private fun level(): String? =
        compose.onNodeWithTag("zoom-level").fetchSemanticsNode().config.getOrNull(SemanticsProperties.StateDescription)

    @Test
    fun theBarIsShutUntilAskedForAndThereIsNothingToZoomWithNoPage() {
        show()
        compose.onNodeWithTag("zoom-level").assertDoesNotExist()
        compose.onNodeWithTag("zoom").assertIsNotEnabled()

        aPageAt(100)
        compose.onNodeWithTag("zoom").assertIsEnabled().performClick()
        compose.waitForIdle()

        assertEquals("100%", level())
        compose.onNodeWithContentDescription("Actual Size").assertExists()
        // At actual size there is nothing to put back.
        compose.onNodeWithTag("zoom-level").assertIsNotEnabled()
        compose.onNodeWithTag("zoom-in").assertIsEnabled()
        compose.onNodeWithTag("zoom-out").assertIsEnabled()
    }

    @Test
    fun theBarSendsWhatIsPressedAndSaysWhatTheCoreSays() {
        show()
        aPageAt(100)
        compose.onNodeWithTag("zoom").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("zoom-in").performClick()
        compose.waitForIdle()
        assertEquals("the bar keeps no level of its own", "100%", level())
        aPageAt(110)
        assertEquals("110%", level())
        compose.onNodeWithTag("zoom-out").performClick()
        compose.onNodeWithTag("zoom-level").assertIsEnabled().performClick()
        compose.waitForIdle()

        val asked = core.requests.mapNotNull { it.zoom_page }
        assertEquals(listOf(ZoomChange.ZOOM_CHANGE_IN, ZoomChange.ZOOM_CHANGE_OUT, ZoomChange.ZOOM_CHANGE_RESET), asked.map { it.change })
        assertEquals(setOf(2L), asked.map { it.tab_id }.toSet())

        compose.onNodeWithTag("zoom-done").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("zoom-level").assertDoesNotExist()
    }

    @Test
    fun theEndsOfTheLevelsCannotBePressed() {
        show()
        aPageAt(300, canZoomIn = false)
        compose.onNodeWithTag("zoom").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("zoom-in").assertIsNotEnabled()
        compose.onNodeWithTag("zoom-out").assertIsEnabled()

        aPageAt(50, canZoomOut = false)
        compose.onNodeWithTag("zoom-out").assertIsNotEnabled()
        compose.onNodeWithTag("zoom-in").assertIsEnabled()
        assertEquals("50%", level())
    }
}
