// A server that asks who is there (017), drawn against a scripted core: the bar, what it shows
// of the core's snapshot, and what each answer sends.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
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
import androidx.compose.ui.text.AnnotatedString
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.PromptView
import com.tigercodes.webtiger.core.v1.SignInPrompt
import com.tigercodes.webtiger.core.v1.WindowSnapshot
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

/** A snapshot of window 1, tab 2, whose page of example.com is left for a server that asks who is there, under the core's name `id`. */
internal fun signingIn(id: Long, site: String = "staff.example.com", confidential: Boolean = true, again: Boolean = false): WindowSnapshot =
    snapshot(url = "https://example.com/", display = "example.com", webView = 4) { tab ->
        tab.copy(prompt = PromptView(prompt_id = id, sign_in = SignInPrompt(site = site, confidential = confidential, again = again)))
    }

@RunWith(RobolectricTestRunner::class)
class SignInBarTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val core = ScriptedCore(::windowScript)

    @After
    fun end() {
        scope.cancel()
    }

    private fun aWindow() {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
    }

    private fun theCoreSays(snapshot: WindowSnapshot) {
        core.emit(snapshot, subscription = 3)
        compose.waitForIdle()
    }

    /** What was sent as answers, each in a few words. */
    private fun answers(): List<String> =
        core.requests.mapNotNull { it.answer_prompt }.map { answer ->
            "prompt ${answer.prompt_id}: " + if (answer.allow) "sign in as \"${answer.text}\" with \"${answer.secret}\"" else "cancel \"${answer.text}\" \"${answer.secret}\""
        }

    @Test
    fun theBarNamesTheServerAndAsksForANameAndAPassword() {
        aWindow()

        theCoreSays(signingIn(9))

        compose.onNodeWithTag("prompt-site").assertTextEquals("Sign in to staff.example.com")
        // Nothing more is said of a first question over a private connection.
        compose.onAllNodesWithTag("prompt-again").assertCountEquals(0)
        compose.onAllNodesWithTag("prompt-not-private").assertCountEquals(0)
        // Two fields, empty, each read by its name; the password's is one.
        compose.onNodeWithContentDescription("Name").assertExists()
        compose.onNodeWithContentDescription("Password").assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Password))
        compose.onNodeWithTag("prompt-name").assert(SemanticsMatcher.keyNotDefined(SemanticsProperties.Password))
        // Nothing has the keyboard by appearing: a key meant for the page must not answer.
        compose.onNodeWithTag("prompt-name").assertIsNotFocused()
        compose.onNodeWithTag("prompt-sign-in").assertIsNotFocused()
        assertEquals(emptyList<String>(), answers())
    }

    @Test
    fun signInSendsTheNameAndThePasswordAndCancelSendsNeither() {
        aWindow()
        theCoreSays(signingIn(9))

        compose.onNodeWithTag("prompt-name").performTextReplacement("ada")
        compose.onNodeWithTag("prompt-password").performTextReplacement("love lace")
        // What is typed for a password is drawn as dots, one for each character.
        compose.onNodeWithTag("prompt-password").assert(SemanticsMatcher.expectValue(SemanticsProperties.EditableText, AnnotatedString("\u2022".repeat(9))))
        compose.onNodeWithTag("prompt-name").assert(SemanticsMatcher.expectValue(SemanticsProperties.EditableText, AnnotatedString("ada")))
        compose.onNodeWithTag("prompt-sign-in").performClick()
        compose.onNodeWithTag("prompt-cancel").performClick()
        // The keyboard's Done, in the password's field, signs in.
        compose.onNodeWithTag("prompt-password").performImeAction()
        compose.waitForIdle()

        assertEquals(
            listOf("prompt 9: sign in as \"ada\" with \"love lace\"", "prompt 9: cancel \"\" \"\"", "prompt 9: sign in as \"ada\" with \"love lace\""),
            answers(),
        )
        // The bar stays until the core says the prompt is gone.
        compose.onAllNodesWithTag("prompt").assertCountEquals(1)

        // Another question starts empty: nothing typed for the last is carried into it.
        theCoreSays(signingIn(10))
        compose.onNodeWithTag("prompt-sign-in").performClick()
        compose.waitForIdle()
        assertEquals("prompt 10: sign in as \"\" with \"\"", answers().last())
    }

    @Test
    fun aServerThatRefusedBeforeAndAConnectionThatIsNotPrivateAreSaid() {
        aWindow()

        theCoreSays(signingIn(9, site = "http://printer.example.com", confidential = false, again = true))

        compose.onNodeWithTag("prompt-site").assertTextEquals("Sign in to http://printer.example.com")
        compose.onNodeWithTag("prompt-again").assertTextEquals("That name or password was not accepted.")
        compose.onNodeWithTag("prompt-not-private").assertTextEquals("This connection is not private. What you type could be read by others.")

        // Each by itself.
        theCoreSays(signingIn(10, again = true))
        compose.onAllNodesWithTag("prompt-again").assertCountEquals(1)
        compose.onAllNodesWithTag("prompt-not-private").assertCountEquals(0)
        theCoreSays(signingIn(11, confidential = false))
        compose.onAllNodesWithTag("prompt-again").assertCountEquals(0)
        compose.onAllNodesWithTag("prompt-not-private").assertCountEquals(1)
    }
}
