// Going on past a certificate that is not trusted (026 §6), drawn against a scripted core: what
// the failed page offers when the core says the load may be gone on with, that nothing goes on
// by one press, what the warning says, and that the warning is one failure's.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

private const val ROUTER = "https://router.example.com/"

@RunWith(RobolectricTestRunner::class)
class CertificateWarningTest {
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

    /** The core says the tab's load failed for a certificate: one that may be gone on past, or not. */
    private fun fails(
        passable: Boolean = true,
        host: String = "router.example.com",
        code: String = "net.tls.untrusted",
        tab: Long = 2,
        namesHost: Boolean = passable,
    ) {
        compose.runOnIdle {
            core.emit(
                snapshot(url = "https://$host/", display = host, state = LoadState.LOAD_STATE_FAILED) {
                    it.copy(tab_id = tab, error_code = code, certificate_passable = passable, certificate_host = if (namesHost) host else "")
                },
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    /** The core says the tab is loading: what a new load, or going on, begins with. */
    private fun loads() {
        compose.runOnIdle { core.emit(snapshot(url = ROUTER, display = "router.example.com", state = LoadState.LOAD_STATE_LOADING, webView = 4), subscription = 3) }
        compose.waitForIdle()
    }

    /** The tabs the core was asked to go on in, in order. */
    private fun goneOn(): List<Long> = core.requests.mapNotNull { it.proceed_past_certificate?.tab_id }

    private fun exists(tag: String): Boolean = compose.onAllNodesWithTag(tag).fetchSemanticsNodes().isNotEmpty()

    @Test
    fun nothingGoesOnByOnePressAndTheWarningComesFirst() {
        show()
        fails()

        // The page says what is wrong, as before, with the way to try again; and under it the
        // way to say more. Nothing on it goes on.
        compose.onNodeWithTag("failed-words").assertTextEquals("This device doesn’t trust this site’s certificate.")
        compose.onNodeWithTag("retry").assertIsDisplayed()
        compose.onNodeWithTag("certificate-advanced").assertTextEquals("Advanced…")
        assertFalse("no way on before the warning", exists("certificate-go-on"))
        assertFalse(exists("certificate-warning"))

        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()

        // The warning: what cannot be confirmed, of whom, and what could come of it; and the
        // one button that goes on, which says that it is not safe.
        assertEquals("the warning asks the core nothing", emptyList<Long>(), goneOn())
        assertTrue(model.isWarnedOfCertificate)
        compose.onNodeWithText("Going on isn’t safe").performScrollTo().assertIsDisplayed()
        compose
            .onNodeWithTag("certificate-warning-words")
            .assertTextEquals(
                "WebTiger can’t confirm that this is really router.example.com. Someone could be pretending to be it, " +
                    "and could read or change what you send and receive: passwords, messages, card numbers.",
            )
        compose.onNodeWithText("Go on only if you know why this site’s certificate isn’t trusted, such as a router or a device of your own.").assertExists()
        compose.onNodeWithTag("certificate-go-on").assertTextEquals("Go On to router.example.com (Not Safe)")
        assertFalse("the warning is in the place of the button that showed it", exists("certificate-advanced"))
        compose.onNodeWithTag("retry").assertExists()

        compose.onNodeWithTag("certificate-go-on").performScrollTo().performClick()
        compose.waitForIdle()

        // The core is asked, for the tab in front, once; what comes of it is the core's.
        assertEquals(listOf(2L), goneOn())
        assertFalse(model.isWarnedOfCertificate)
    }

    @Test
    fun aFailureTheCoreDoesNotSayMayBeGoneOnWithOffersNoWayOn() {
        show()
        // A certificate's trouble, but not one to go on past (the adapter holds nothing), and
        // a failure that is no certificate's.
        fails(passable = false)
        compose.onNodeWithTag("failed-page").assertIsDisplayed()
        assertFalse(exists("certificate-advanced"))
        assertFalse(exists("certificate-go-on"))
        fails(passable = false, code = "net.dns")
        assertFalse(exists("certificate-advanced"))

        // And the model asks the core nothing, whoever calls it.
        compose.runOnIdle {
            model.warnOfCertificate()
            model.goOnPastCertificate()
        }
        assertFalse(model.isWarnedOfCertificate)
        assertEquals(emptyList<Long>(), goneOn())
    }

    @Test
    fun withoutTheWarningOnTheScreenTheCoreIsNotAsked() {
        show()
        fails()

        compose.runOnIdle { model.goOnPastCertificate() }

        assertEquals("going on is only ever the warning's button", emptyList<Long>(), goneOn())
    }

    @Test
    fun theWarningIsOneFailuresAndAnotherStartsAgain() {
        show()
        fails()
        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()
        assertTrue(exists("certificate-go-on"))

        // The same failure said again (a snapshot for something else) leaves the warning open.
        fails()
        assertTrue(exists("certificate-go-on"))

        // Another load of the tab, which fails the same way: the warning is not the new failure's.
        loads()
        assertFalse(exists("certificate-go-on"))
        fails()
        assertFalse(exists("certificate-go-on"))
        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()
        assertTrue(exists("certificate-go-on"))

        // A failure there that may not be gone on with ends it too, whatever host a core names.
        fails(passable = false, namesHost = true)
        assertFalse(exists("certificate-advanced"))
        fails()
        assertFalse("the warning was the failure before's", exists("certificate-go-on"))
        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()

        // Another site's certificate, in the same tab.
        fails(host = "printer.example.com")
        assertFalse("the warning named another site", exists("certificate-go-on"))
        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("certificate-go-on").assertTextEquals("Go On to printer.example.com (Not Safe)")

        // Another tab in front, with the same trouble.
        fails(host = "printer.example.com", tab = 7)
        assertFalse("the warning was another tab's", exists("certificate-go-on"))
        assertEquals(emptyList<Long>(), goneOn())
    }

    @Test
    fun tryingAgainIsStillThere() {
        show()
        fails()
        compose.onNodeWithTag("certificate-advanced").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("retry").performScrollTo().performClick()
        compose.waitForIdle()

        assertEquals(NavigationCommand.NAVIGATION_COMMAND_RELOAD, core.requests.last().tab_command?.command)
        assertEquals(emptyList<Long>(), goneOn())
    }
}
