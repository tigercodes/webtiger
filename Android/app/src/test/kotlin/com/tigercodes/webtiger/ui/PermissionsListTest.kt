// The Permissions list (014), over a scripted core: what its model asks and sends, what the list
// shows, and the window's way into it.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.DownloadsPage
import com.tigercodes.webtiger.core.v1.HistoryPage
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_CAMERA
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_DOWNLOADS
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_LOCATION
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_MICROPHONE
import com.tigercodes.webtiger.core.v1.PermissionsPage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SitePermissions
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** A core with sites that were allowed things, which takes back what it is told to. */
internal class PermissionsScript {
    var sites: List<SitePermissions> =
        listOf(
            SitePermissions(origin = "http://127.0.0.1:8080", site = "http://127.0.0.1:8080", kinds = listOf(PERMISSION_KIND_LOCATION, PERMISSION_KIND_DOWNLOADS)),
            SitePermissions(origin = "https://meet.example.com", site = "meet.example.com", kinds = listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE)),
        )

    fun answer(request: Request): Response {
        val forget = request.forget_permissions
        if (forget != null) sites = if (forget.everything == true) emptyList() else sites.filter { it.origin != forget.origin }
        return when {
            request.query_permissions != null -> Response(permissions_page = PermissionsPage(sites = sites))
            request.query_history != null -> Response(history_page = HistoryPage())
            request.query_downloads != null -> Response(downloads_page = DownloadsPage())
            else -> windowScript(request)
        }
    }
}

class PermissionsModelTest {
    private fun names(model: PermissionsModel) = model.sites.map { it.site }

    @Test
    fun theListIsWhatTheCoreSaysOfTheWindowsSites() =
        runTest {
            val core = ScriptedCore(PermissionsScript()::answer)
            val model = PermissionsModel(unconfinedSession(core), windowId = 7)
            assertFalse("nothing is said before the core has answered", model.loaded)

            model.reload()

            assertEquals(7L, core.requests.last().query_permissions?.window_id)
            assertTrue(model.loaded)
            assertEquals("in the core's order", listOf("http://127.0.0.1:8080", "meet.example.com"), names(model))
        }

    @Test
    fun takingBackOneLeavesTheListAtOnceAndTakingBackAllAsksAgain() =
        runTest {
            val core = ScriptedCore(PermissionsScript()::answer)
            val model = PermissionsModel(unconfinedSession(core), windowId = 7)
            model.reload()

            model.forget(model.sites[1])

            val forget = core.requests.last().forget_permissions
            assertEquals(7L, forget?.window_id)
            assertEquals("named by its origin, as the core gave it", "https://meet.example.com", forget?.origin)
            assertEquals("it has left, with nothing asked", listOf("http://127.0.0.1:8080"), names(model))

            model.forgetEverything()

            val sent = core.requests.takeLast(2)
            assertEquals(true, sent[0].forget_permissions?.everything)
            assertNotNull("what is left is asked for", sent[1].query_permissions)
            assertTrue(model.sites.isEmpty())
            assertTrue("the core has answered: no site is allowed anything", model.loaded)
        }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun onlyTheAnswerToTheLastQuestionIsShown() =
        runTest {
            val script = PermissionsScript()
            val core = HeldAnswers(ScriptedCore(script::answer))
            val session = BrowserSession(core, SystemWebViewEngine(FakePages()), CoroutineScope(backgroundScope.coroutineContext + UnconfinedTestDispatcher(testScheduler)))
            val model = PermissionsModel(session, windowId = 7)

            // The first question's answer is made, and held; by the second, a site has gone.
            model.reload()
            script.sites = script.sites.drop(1)
            model.reload()
            core.held[1].complete(Unit)
            assertEquals(listOf("meet.example.com"), names(model))
            core.held[0].complete(Unit)

            assertEquals("an answer to a question that was asked again is out of date", listOf("meet.example.com"), names(model))
        }
}

/** A core whose answer about permissions waits until the test lets it go: answers that come in any order. */
private class HeldAnswers(private val core: ScriptedCore) : CoreClient by core {
    val held = ArrayList<CompletableDeferred<Unit>>()

    override suspend fun request(request: Request): Response {
        val answer = core.request(request)
        if (request.query_permissions != null) CompletableDeferred<Unit>().also { held.add(it) }.await()
        return answer
    }
}

@RunWith(RobolectricTestRunner::class)
class PermissionsListTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val script = PermissionsScript()
    private val core = ScriptedCore(script::answer)

    @After
    fun end() {
        scope.cancel()
    }

    private fun session() = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)

    private var done = 0

    private fun show() {
        val model = PermissionsModel(session(), windowId = 1)
        compose.setContent { WebTigerTheme { PermissionsList(model, onDone = { done += 1 }) } }
        compose.waitForIdle()
    }

    private fun asked(): Int = core.requests.count { it.query_permissions != null }

    @Test
    fun aSiteIsShownByItsNameAndWhatItMayUse() {
        show()
        compose.onAllNodesWithTag("permission-site").assertCountEquals(2)
        val device = compose.onAllNodesWithTag("permission-site")[0]
        device.assertTextContains("http://127.0.0.1:8080")
        device.assertTextContains("Location and Downloads")
        val meet = compose.onAllNodesWithTag("permission-site")[1]
        meet.assertTextContains("meet.example.com")
        meet.assertTextContains("Camera and Microphone")
        // Each has the button that takes it back, named for what it does.
        compose.onAllNodesWithTag("permission-forget").assertCountEquals(2)
        compose.onAllNodesWithContentDescription("Stop Allowing").assertCountEquals(2)
        assertEquals("asked once, when it is shown", 1, asked())
    }

    @Test
    fun theButtonBesideASiteTakesBackWhatItWasAllowed() {
        show()

        compose.onAllNodesWithTag("permission-forget")[1].performClick()
        compose.waitForIdle()

        assertEquals("https://meet.example.com", core.requests.last().forget_permissions?.origin)
        compose.onAllNodesWithTag("permission-site").assertCountEquals(1)
        compose.onAllNodesWithTag("permission-site")[0].assertTextContains("http://127.0.0.1:8080")
    }

    @Test
    fun removeAllTakesEverythingBackAndThenThereIsNothingToRemove() {
        show()
        compose.onNodeWithTag("permissions-nothing").assertDoesNotExist()
        compose.onNodeWithTag("permissions-clear").assertIsEnabled().performClick()
        compose.waitForIdle()

        assertEquals(listOf(true), core.requests.mapNotNull { it.forget_permissions }.map { it.everything })
        compose.onAllNodesWithTag("permission-site").assertCountEquals(0)
        compose.onNodeWithTag("permissions-nothing").assertTextContains("Sites you allow to download, or to use your camera, microphone or location, will be listed here.")
        compose.onNodeWithTag("permissions-clear").assertIsNotEnabled()
        compose.onNodeWithTag("permissions-done").performClick()
        assertEquals(1, done)
    }

    // The window's way into the list

    @Test
    fun theSwitchHasAThirdListAndNothingIsAskedOfItUntilItIsShown() {
        val session = session()
        val model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()

        compose.onNodeWithContentDescription("History and Downloads").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("library-history").assertIsSelected()
        assertEquals(0, asked())

        compose.onNodeWithTag("library-permissions").performClick()
        compose.waitForIdle()

        assertEquals("the window's own sites are asked for", 1L, core.requests.last { it.query_permissions != null }.query_permissions?.window_id)
        compose.onNodeWithTag("library-permissions").assertIsSelected()
        compose.onAllNodesWithTag("permission-site").assertCountEquals(2)
        compose.onNodeWithTag("history-search").assertDoesNotExist()

        compose.onNodeWithTag("permissions-done").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("library-permissions").assertDoesNotExist()
    }
}
