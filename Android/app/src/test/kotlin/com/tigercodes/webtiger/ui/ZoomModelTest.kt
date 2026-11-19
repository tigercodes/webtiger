// The window's model and zoom (012), against a scripted core: what it asks, what its bar says,
// and when the bar is open.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.core.v1.ZoomChange
import com.tigercodes.webtiger.core.v1.ZoomView
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ZoomModelTest {
    private suspend fun TestScope.opened(core: ScriptedCore): BrowserModel = BrowserModel(unconfinedSession(core)).also { it.open() }

    /**
     * A window's snapshot: the tab in front on a page, zoomed as the core would say, or with
     * `percent` null on no page at all.
     */
    private fun window(front: Long = 2, percent: Int?, canZoomIn: Boolean = true, canZoomOut: Boolean = true): WindowSnapshot =
        if (percent == null) {
            snapshot { it.copy(tab_id = front) }
        } else {
            snapshot(url = "https://example.com/", display = "example.com", webView = 4) {
                it.copy(tab_id = front, zoom = ZoomView(percent = percent, can_zoom_in = canZoomIn, can_zoom_out = canZoomOut))
            }
        }

    private fun zooms(core: ScriptedCore): List<String> = core.requests.mapNotNull { it.zoom_page }.map { "${it.tab_id} ${it.change}" }

    @Test
    fun thereIsNothingToZoomWithoutAPage() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = opened(core)
            assertNull(model.zoom)

            model.openZoom()
            model.zoom(ZoomChange.ZOOM_CHANGE_IN)

            assertFalse("the bar stays shut", model.isZooming)
            assertEquals("and nothing is asked", emptyList<String>(), zooms(core))
        }

    @Test
    fun biggerSmallerAndActualSizeAskTheCoreForTheTabInFront() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = opened(core)
            core.emit(window(front = 9, percent = 100), subscription = 3)

            model.zoom(ZoomChange.ZOOM_CHANGE_IN)
            model.zoom(ZoomChange.ZOOM_CHANGE_OUT)
            model.zoom(ZoomChange.ZOOM_CHANGE_RESET)

            assertEquals(
                "which level each is, is the core's",
                listOf("9 ZOOM_CHANGE_IN", "9 ZOOM_CHANGE_OUT", "9 ZOOM_CHANGE_RESET"),
                zooms(core),
            )
        }

    @Test
    fun theBarSaysWhatTheCoreSays() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = opened(core)
            core.emit(window(percent = 100), subscription = 3)
            model.openZoom()
            assertTrue(model.isZooming)
            assertEquals(100, model.zoom?.percent)

            // The model keeps no level of its own: asking changes nothing until the core says.
            model.zoom(ZoomChange.ZOOM_CHANGE_IN)
            assertEquals(100, model.zoom?.percent)
            core.emit(window(percent = 300, canZoomIn = false), subscription = 3)

            assertEquals(300, model.zoom?.percent)
            assertEquals("nothing above the largest: the core says so", false, model.zoom?.can_zoom_in)
            assertEquals(true, model.zoom?.can_zoom_out)
        }

    @Test
    fun oneBarAtATime() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = opened(core)
            core.emit(window(percent = 110), subscription = 3)

            model.openFind()
            model.openZoom()
            assertTrue(model.isZooming)
            assertFalse("the zoom bar opens in place of the find bar", model.isFinding)

            model.openFind()
            assertTrue(model.isFinding)
            assertFalse("and the find bar in place of the zoom bar", model.isZooming)

            model.openZoom()
            model.closeZoom()
            assertFalse(model.isZooming)
        }

    @Test
    fun theBarClosesWhenThePageInFrontCannotBeZoomed() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = opened(core)
            core.emit(window(percent = 125), subscription = 3)
            model.openZoom()

            // Another tab's page in front: the bar is the window's, and says that page's level.
            core.emit(window(front = 9, percent = 90), subscription = 3)
            assertTrue(model.isZooming)
            assertEquals(90, model.zoom?.percent)

            // An empty tab in front: nothing to zoom, and nothing for the bar to say.
            core.emit(window(front = 11, percent = null), subscription = 3)
            assertFalse(model.isZooming)
            assertNull(model.zoom)
        }
}
