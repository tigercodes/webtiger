// Passwords on Android (032), in the app itself on a device, in fake mode: the system's Autofill
// service is asked about a page's sign-in form in WebTiger's window when a finger puts the
// cursor in it, and is told whose form it is and what its fields are for. The service is the
// tests' own, set as the device's for the test and the device's own put back after it; it
// offers nothing, so nothing of the system's is shown.
package com.tigercodes.webtiger

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.ParcelFileDescriptor
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppAutofillTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>
    private val context: Context get() = ApplicationProvider.getApplicationContext()
    private val site get() = AppWindowTest.site

    /** The Autofill service the device had before the test: put back when it ends. */
    private var devices = ""

    /** Each thing the tests' service said it was asked about, in order: the site, its scheme, and the fields. */
    private val told = java.util.concurrent.CopyOnWriteArrayList<Triple<String, String, List<String>>>()

    private val hears =
        object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                told +=
                    Triple(
                        intent.getStringExtra(AskedAutofillService.DOMAIN).orEmpty(),
                        intent.getStringExtra(AskedAutofillService.SCHEME).orEmpty(),
                        intent.getStringArrayListExtra(AskedAutofillService.FIELDS).orEmpty(),
                    )
            }
        }

    /** Runs a command as the device's shell, and what it printed. */
    private fun shell(command: String): String {
        val output = InstrumentationRegistry.getInstrumentation().uiAutomation.executeShellCommand(command)
        return ParcelFileDescriptor.AutoCloseInputStream(output).use { it.readBytes().decodeToString() }.trim()
    }

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
        devices = shell("settings get secure autofill_service")
        shell("settings put secure autofill_service ${AskedAutofillService.COMPONENT}")
        context.registerReceiver(hears, IntentFilter(AskedAutofillService.ASKED), Context.RECEIVER_EXPORTED)
        val intent =
            Intent(context, MainActivity::class.java)
                .putExtra(MainActivity.EXTRA_FAKE, true)
                // The process's one start address: the app's core starts once, and every window
                // of every test after this one opens on it.
                .putExtra(MainActivity.EXTRA_FAKE_START, site.url("/"))
        scenario = ActivityScenario.launch(intent)
    }

    @After
    fun close() {
        scenario.close()
        context.unregisterReceiver(hears)
        // The device's own service again, or none where it had none.
        if (devices.isEmpty() || devices == "null") shell("settings delete secure autofill_service") else shell("settings put secure autofill_service $devices")
    }

    private fun webViewIn(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewIn(view.getChildAt(index))?.let { return it }
        }
        return null
    }

    /** Waits for the page titled `title` to be the one the window shows. */
    private fun awaitPage(title: String) {
        fun shownTitle(): String {
            var now = ""
            scenario.onActivity { activity -> now = webViewIn(activity.window.decorView)?.title.orEmpty() }
            return now
        }
        try {
            compose.waitUntil(15_000) { shownTitle() == title }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // A test site's page on the loopback address, not anyone's browsing.
            throw AssertionError("the page <$title> never showed; the window shows <${shownTitle()}>")
        }
    }

    /** The WebView the window shows. */
    private fun shown(): WebView {
        var found: WebView? = null
        scenario.onActivity { activity -> found = webViewIn(activity.window.decorView) }
        return checkNotNull(found) { "the window shows no web view" }
    }

    @Test
    fun theSystemsAutofillIsAskedAboutAPagesSignInFormWithItsSiteAndItsFields() {
        awaitPage("Home")
        // The page with the sign-in form, typed in the address field.
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url("/sign-in-form"))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage("Sign in")
        assertEquals("the tests' service is the device's for now", AskedAutofillService.COMPONENT, shell("settings get secure autofill_service"))

        // A finger puts the cursor in the field for a name.
        PageTouch.tap(shown(), "user")

        // The service is asked about the form: the request that has the form's fields in it.
        // (It may have been asked about the window's own address field before, when the
        // address was typed: that one has no form in it, and no site.)
        try {
            compose.waitUntil(20_000) { told.any { asked -> asked.third.isNotEmpty() } }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // What a test's own page said of its form, never anybody's.
            throw AssertionError("the system's Autofill service was never asked about the form; it was asked about $told")
        }
        val (domain, scheme, fields) = told.first { asked -> asked.third.isNotEmpty() }
        // Whose form it is: the page's own site, as the WebView says it and not the page.
        assertEquals("all it was asked: $told", "127.0.0.1", domain)
        assertEquals("http", scheme)
        // And what its fields are for, in the page's own words for it: a name, with the cursor
        // in it, and a password. Each once: the form is described to the service one time
        // (ui/FillableWindow.kt says when it would be twice).
        assertEquals(listOf("username/text/username/focused", "password/password/current-password/"), fields)
        // Asked once about the form: putting the cursor there is one question.
        assertEquals(1, told.count { asked -> asked.third.isNotEmpty() })
    }
}
