// A Space's icon (050) in the app itself on a device, in fake mode: the real core, the real
// WebView, the window on a real screen. The page that edits the Space the window shows offers
// the icons the core lists; one that is chosen is the Space's, and the chip draws it; given
// none again, the chip has none. The chip is said by the Space's name throughout.
//
// (The Space is left as it was found: the device suite's tests share one core.)
package com.tigercodes.webtiger

import android.content.Intent
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
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
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppSpaceIconTest {
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

    /** The icons the chip draws: its own, when the Space has one. */
    private fun chipIcons(): List<String> =
        compose
            .onAllNodesWithTag("space-icon-mark", useUnmergedTree = true)
            .fetchSemanticsNodes()
            .mapNotNull { it.config.getOrNull(SemanticsProperties.Text)?.joinToString("") { text -> text.text } }

    /** What the chip says to a screen reader. */
    private fun chipSays(): String =
        compose
            .onNodeWithTag("space-chip")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.ContentDescription)
            ?.joinToString()
            .orEmpty()

    @Test
    fun theSpaceShownIsGivenAnIconWhichTheChipDrawsAndThenNoneAgain() {
        compose.enableAccessibilityChecks()
        try {
            awaitTag("space-chip")
            val said = chipSays()
            assertTrue("the chip names its Space: <$said>", said.startsWith("Space: "))

            // The page that edits the Space: none, and the icons the real core lists.
            editSpace()
            compose.onNodeWithTag("space-icon").performScrollTo().assertTextEquals("Icon")
            compose.onNodeWithTag("space-icon-none").performScrollTo().assertIsSelected()
            compose.onNodeWithTag("space-icon-24").performScrollTo().assertIsNotSelected()
            compose.onAllNodesWithTag("space-icon-25").assertCountEquals(0)
            compose.onRoot().tryPerformAccessibilityChecks()
            // What the second is, as this device draws it: the chip is to draw the same.
            val second =
                compose
                    .onNodeWithTag("space-icon-2")
                    .performScrollTo()
                    .fetchSemanticsNode()
                    .config
                    .getOrNull(SemanticsProperties.Text)
                    ?.joinToString("") { it.text }
                    .orEmpty()
            assertTrue("an icon is not words: <${second.codePoints().toArray().joinToString("+") { it.toString(16) }}>", second.isNotEmpty() && second.none { it.code < 128 })

            compose.onNodeWithTag("space-icon-2").performClick()
            compose.onNodeWithTag("space-icon-2").assertIsSelected()
            compose.onNodeWithTag("space-icon-none").performScrollTo().assertIsNotSelected()
            save()

            // The chip draws it, and is said by the Space's name as before.
            compose.waitUntil(15_000) { chipIcons() == listOf(second) }
            assertEquals(said, chipSays())
            // And the page says it is the Space's.
            editSpace()
            compose.onNodeWithTag("space-icon-2").performScrollTo().assertIsSelected()

            // None again: the Space is left as it was found, and the chip has no icon.
            compose.onNodeWithTag("space-icon-none").performScrollTo().performClick()
            save()
            compose.waitUntil(15_000) { chipIcons().isEmpty() }
            assertEquals(said, chipSays())
        } finally {
            compose.disableAccessibilityChecks()
        }
    }
}
