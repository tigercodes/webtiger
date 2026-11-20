// What a tab asks the person (014), against a scripted core: when the bar is there, what it
// says, and what its buttons send.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_CAMERA
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_LOCATION
import com.tigercodes.webtiger.core.v1.PermissionKind.PERMISSION_KIND_MICROPHONE
import com.tigercodes.webtiger.core.v1.PermissionPrompt
import com.tigercodes.webtiger.core.v1.PromptView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** A snapshot of window 1, tab 2, whose page asks for these, as `site`, under the core's name `id`. */
internal fun asking(id: Long, vararg kinds: PermissionKind, site: String = "meet.example.com"): WindowSnapshot =
    snapshot(url = "https://meet.example.com/", display = "meet.example.com", webView = 4) { tab ->
        tab.copy(prompt = PromptView(prompt_id = id, permission = PermissionPrompt(site = site, kinds = kinds.toList())))
    }

@RunWith(RobolectricTestRunner::class)
class PromptTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val core = ScriptedCore(::windowScript)

    @After
    fun end() {
        scope.cancel()
    }

    private lateinit var model: BrowserModel

    private fun aWindow() {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
    }

    private fun theCoreSays(snapshot: WindowSnapshot) {
        core.emit(snapshot, subscription = 3)
        compose.waitForIdle()
    }

    private fun answers() = core.requests.mapNotNull { it.answer_prompt }

    @Test
    fun theBarIsThereWhileTheTabAsksAndGoneWhenItDoesNot() {
        aWindow()
        compose.onAllNodesWithTag("prompt").assertCountEquals(0)
        assertNull(model.prompt)

        theCoreSays(asking(9, PERMISSION_KIND_CAMERA))

        compose.onAllNodesWithTag("prompt").assertCountEquals(1)
        assertEquals(9L, model.prompt?.prompt_id)
        compose.onNodeWithTag("prompt-words").assertTextEquals("meet.example.com wants to use your camera.")

        // Answered, or its page gone: the core's next snapshot has no prompt.
        theCoreSays(snapshot(url = "https://meet.example.com/", display = "meet.example.com", webView = 4))
        compose.onAllNodesWithTag("prompt").assertCountEquals(0)
    }

    @Test
    fun theSentenceNamesWhoAsksInTheCoresWordsAndWhatFor() {
        aWindow()
        for ((kinds, words) in listOf(
            listOf(PERMISSION_KIND_MICROPHONE) to "meet.example.com wants to use your microphone.",
            listOf(PERMISSION_KIND_LOCATION) to "meet.example.com wants to use your location.",
            listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE) to "meet.example.com wants to use your camera and your microphone.",
            listOf(PERMISSION_KIND_CAMERA, PERMISSION_KIND_MICROPHONE, PERMISSION_KIND_LOCATION) to
                "meet.example.com wants to use your camera, your microphone, and your location.",
        )) {
            theCoreSays(asking(9, *kinds.toTypedArray()))
            compose.onNodeWithTag("prompt-words").assertTextEquals(words)
        }
        // Who asks is what the core says, as it says it: a page not over https has its scheme.
        theCoreSays(asking(9, PERMISSION_KIND_CAMERA, site = "http://127.0.0.1:8080"))
        compose.onNodeWithTag("prompt-words").assertTextEquals("http://127.0.0.1:8080 wants to use your camera.")
    }

    @Test
    fun eachButtonSendsItsAnswerForThePromptThatIsShown() {
        aWindow()
        theCoreSays(asking(9, PERMISSION_KIND_CAMERA))

        compose.onNodeWithTag("prompt-refuse").performClick()
        compose.waitForIdle()

        assertEquals(listOf(Triple(2L, 9L, false)), answers().map { Triple(it.tab_id, it.prompt_id, it.allow) })
        // The bar stays until the core says the prompt is gone: an answer that came too soon
        // answered nothing, and the next one shown has a name of its own.
        compose.onAllNodesWithTag("prompt").assertCountEquals(1)
        theCoreSays(asking(12, PERMISSION_KIND_LOCATION))
        compose.onNodeWithTag("prompt-allow").performClick()
        compose.waitForIdle()
        assertEquals(Triple(2L, 12L, true), answers().last().let { Triple(it.tab_id, it.prompt_id, it.allow) })
        assertEquals(2, answers().size)
    }

    @Test
    fun theBarTakesTheKeyboardFromNothingAndNoButtonHasIt() {
        aWindow()
        theCoreSays(asking(9, PERMISSION_KIND_CAMERA))

        // A key meant for the page or the field must not answer for the person.
        compose.onNodeWithTag("prompt-allow").assertIsNotFocused()
        compose.onNodeWithTag("prompt-refuse").assertIsNotFocused()
        assertEquals(0, answers().size)
    }

    @Test
    fun aPromptThisBuildHasNoWordsForIsNotPutBeforeAPerson() {
        aWindow()

        // A prompt of a kind a later core knows, and one that names nothing this build knows.
        theCoreSays(snapshot(url = "https://meet.example.com/", display = "meet.example.com", webView = 4) { it.copy(prompt = PromptView(prompt_id = 9)) })
        compose.onAllNodesWithTag("prompt").assertCountEquals(0)
        theCoreSays(asking(10))
        compose.onAllNodesWithTag("prompt").assertCountEquals(0)
        assertNull(model.prompt)

        // Nothing is sent for what is not shown.
        model.answerPrompt(true)
        assertEquals(0, answers().size)
        theCoreSays(asking(11, PERMISSION_KIND_CAMERA))
        assertNotNull(model.prompt)
    }
}
