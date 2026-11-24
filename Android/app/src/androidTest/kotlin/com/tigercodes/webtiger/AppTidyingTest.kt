// Tidying (049) in the app itself on a device, in fake mode: the real core, the real WebView,
// the window on a real screen. The page that edits a Space has the times its tabs can be tidied
// after, with never chosen; a time that is chosen is the Space's when the page is opened again;
// and the Library has the Archive, which says that nothing has been tidied.
//
// (No tab is tidied here. The device suite's tests share one core, whose clock moves a second
// each time it is read, and twelve hours are the soonest a tab is tidied. A tab is tidied by
// the real core under this shell on the JVM, with the clock a day a read: ThroughTheCoreTest.)
package com.tigercodes.webtiger

import android.content.Intent
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.accessibility.disableAccessibilityChecks
import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppTidyingTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>

    /** The process's one site: the app's core starts once, and every window opens on it. */
    private val site get() = AppWindowTest.site

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
        val intent =
            Intent(ApplicationProvider.getApplicationContext(), MainActivity::class.java)
                .putExtra(MainActivity.EXTRA_FAKE, true)
                .putExtra(MainActivity.EXTRA_FAKE_START, site.url("/"))
        scenario = ActivityScenario.launch(intent)
    }

    @After
    fun close() {
        scenario.close()
    }

    private fun awaitTag(tag: String) {
        try {
            compose.waitUntil(15_000) { compose.onAllNodesWithTag(tag).fetchSemanticsNodes().isNotEmpty() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("<$tag> never showed")
        }
    }

    private fun awaitGone(tag: String) {
        try {
            compose.waitUntil(15_000) { compose.onAllNodesWithTag(tag).fetchSemanticsNodes().isEmpty() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("<$tag> never went")
        }
    }

    /** Opens the page that edits the Space the window shows. */
    private fun editSpace() {
        awaitTag("space-chip")
        compose.onNodeWithTag("space-chip").performClick()
        awaitTag("spaces-rename")
        compose.onNodeWithTag("spaces-rename").performClick()
        awaitTag("space-save")
    }

    private fun save() {
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        awaitGone("spaces")
    }

    @Test
    fun aSpaceIsGivenItsTimeWhereItIsEditedAndTheLibraryHasItsArchive() {
        compose.enableAccessibilityChecks()
        try {
            // The page that edits the Space: the times there are, and never, as a Space is made.
            editSpace()
            compose.onNodeWithTag("space-tidy").performScrollTo().assertTextEquals("Tidy tabs away")
            for ((time, words) in listOf(0 to "Never", 43_200 to "After 12 hours", 86_400 to "After a day", 604_800 to "After a week", 2_592_000 to "After 30 days")) {
                compose.onNodeWithTag("space-tidy-$time").performScrollTo().assertTextEquals(words)
            }
            compose.onNodeWithTag("space-tidy-0").performScrollTo().assertIsSelected()
            compose.onRoot().tryPerformAccessibilityChecks()

            // Twelve hours: the one pressed is the one chosen; and saved, it is the Space's.
            compose.onNodeWithTag("space-tidy-43200").performScrollTo().performClick()
            compose.onNodeWithTag("space-tidy-43200").assertIsSelected()
            compose.onNodeWithTag("space-tidy-0").assertIsNotSelected()
            save()
            editSpace()
            compose.onNodeWithTag("space-tidy-43200").performScrollTo().assertIsSelected()
            compose.onNodeWithTag("space-tidy-0").assertIsNotSelected()

            // Never again: the Space is left as it was found, for the tests after this one.
            compose.onNodeWithTag("space-tidy-0").performScrollTo().performClick()
            save()
            editSpace()
            compose.onNodeWithTag("space-tidy-0").performScrollTo().assertIsSelected()
            compose.onNodeWithTag("space-back").performScrollTo().performClick()
            awaitTag("spaces-done")
            compose.onNodeWithTag("spaces-done").performClick()
            awaitGone("spaces")

            // The Library's fourth list: nothing has been tidied, and it says so.
            compose.onNodeWithTag("history").performClick()
            awaitTag("library-archive")
            compose.onNodeWithTag("library-archive").performScrollTo().performClick()
            awaitTag("archive-nothing")
            compose.onNodeWithTag("archive-nothing").assertTextEquals("Tabs that are tidied away will be listed here.")
            compose.onNodeWithTag("archive-clear").assertIsNotEnabled()
            compose.onAllNodesWithTag("archive-entry").assertCountEquals(0)
            compose.onRoot().tryPerformAccessibilityChecks()
            compose.onNodeWithTag("archive-done").performClick()
            awaitGone("archive-done")
        } finally {
            compose.disableAccessibilityChecks()
        }
    }
}
