// What a page says or asks (016), drawn against a scripted core: the bar for each of the three,
// what it shows of the core's snapshot, and what each answer sends.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_ALERT
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_CONFIRM
import com.tigercodes.webtiger.core.v1.DialogKind.DIALOG_KIND_PROMPT
import com.tigercodes.webtiger.core.v1.DialogPrompt
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
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** A snapshot of window 1, tab 2, whose page of example.com says `message`, or asks, under the core's name `id`. */
internal fun saying(id: Long, kind: DialogKind, message: String, offered: String = "", maySilence: Boolean = false): WindowSnapshot =
    snapshot(url = "https://example.com/", display = "example.com", webView = 4) { tab ->
        tab.copy(prompt = PromptView(prompt_id = id, dialog = DialogPrompt(site = "example.com", kind = kind, message = message, default_text = offered, may_silence = maySilence)))
    }

@RunWith(RobolectricTestRunner::class)
class DialogBarTest {
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

    /** What was sent as answers, each in a few words. */
    private fun answers(): List<String> =
        core.requests.mapNotNull { it.answer_prompt }.map { answer ->
            buildString {
                append("prompt ${answer.prompt_id}: ")
                append(if (answer.silence) "stop" else if (answer.allow) "ok" else "cancel")
                if (answer.text.isNotEmpty()) append(" \"${answer.text}\"")
            }
        }

    @Test
    fun somethingSaidIsShownUnderTheSitesNameAndIsOnlyAcknowledged() {
        aWindow()

        theCoreSays(saying(9, DIALOG_KIND_ALERT, "Your basket is empty."))

        compose.onNodeWithTag("prompt-site").assertTextEquals("example.com says:")
        compose.onNodeWithTag("prompt-words").assertTextEquals("Your basket is empty.")
        // Nothing to refuse, no field, and a page's first is not to be stopped.
        compose.onAllNodesWithTag("prompt-cancel").assertCountEquals(0)
        compose.onAllNodesWithTag("prompt-field").assertCountEquals(0)
        compose.onAllNodesWithTag("prompt-silence").assertCountEquals(0)
        // No button has the keyboard: a key meant for the page must not answer.
        compose.onNodeWithTag("prompt-ok").assertIsNotFocused()

        compose.onNodeWithTag("prompt-ok").performClick()
        compose.waitForIdle()

        assertEquals(listOf("prompt 9: ok"), answers())
        // The bar stays until the core says the prompt is gone.
        compose.onAllNodesWithTag("prompt").assertCountEquals(1)
    }

    @Test
    fun aQuestionHasTwoAnswersAndAPageThatSaidSomethingBeforeCanBeStopped() {
        aWindow()
        theCoreSays(saying(9, DIALOG_KIND_CONFIRM, "Delete it?", maySilence = true))

        compose.onNodeWithTag("prompt-words").assertTextEquals("Delete it?")
        compose.onNodeWithTag("prompt-cancel").performClick()
        compose.onNodeWithTag("prompt-ok").performClick()
        compose.onNodeWithTag("prompt-silence").performClick()
        compose.waitForIdle()

        assertEquals(listOf("prompt 9: cancel", "prompt 9: ok", "prompt 9: stop"), answers())

        // The page's first question has no such button.
        theCoreSays(saying(10, DIALOG_KIND_CONFIRM, "Sure?"))
        compose.onAllNodesWithTag("prompt-silence").assertCountEquals(0)
    }

    @Test
    fun aPromptHasAFieldWithWhatThePageOffersAndOkSendsWhatIsInIt() {
        aWindow()
        theCoreSays(saying(9, DIALOG_KIND_PROMPT, "What should we call you?", offered = "Ada"))

        compose.onNodeWithContentDescription("Your answer").assertExists()
        compose.onNodeWithTag("prompt-field").assertTextEquals("Ada")
        // Unchanged, OK sends what was offered.
        compose.onNodeWithTag("prompt-ok").performClick()
        compose.onNodeWithTag("prompt-field").performTextReplacement("Grace Hopper")
        compose.onNodeWithTag("prompt-ok").performClick()
        // Cancel sends nothing of what was typed.
        compose.onNodeWithTag("prompt-cancel").performClick()
        // The keyboard's Done, in the field, is OK.
        compose.onNodeWithTag("prompt-field").performImeAction()
        compose.waitForIdle()

        assertEquals(
            listOf("prompt 9: ok \"Ada\"", "prompt 9: ok \"Grace Hopper\"", "prompt 9: cancel", "prompt 9: ok \"Grace Hopper\""),
            answers(),
        )

        // Another prompt starts from what it offers, not from what was typed in the last.
        theCoreSays(saying(10, DIALOG_KIND_PROMPT, "And your town?", offered = ""))
        compose.onNodeWithTag("prompt-field").assertTextEquals("")
        compose.onNodeWithTag("prompt-ok").performClick()
        compose.waitForIdle()
        assertEquals("a field left empty is an answer too", "prompt 10: ok", answers().last())
    }

    @Test
    fun somethingThisBuildHasNoBarForIsNotShown() {
        aWindow()

        theCoreSays(saying(9, DialogKind.DIALOG_KIND_UNSPECIFIED, "Hello"))

        compose.onAllNodesWithTag("prompt").assertCountEquals(0)
        assertNull(model.prompt)
        model.answerPrompt(true)
        assertEquals(emptyList<String>(), answers())
    }
}
