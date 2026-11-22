// Groups (041) in the app, on a device, in fake mode: the real core and the real WebView. A tab
// is put in a new Group from its own menu and is kept by it; the Group's chip shuts it and
// opens it; the Group is edited; the Site is taken out and put back; and the Group is removed,
// which leaves the Site.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.performTouchInput
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
internal class AppGroupsTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>
    private val context: Context get() = ApplicationProvider.getApplicationContext()
    private val site get() = AppWindowTest.site

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
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
    }

    private fun webViewIn(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewIn(view.getChildAt(index))?.let { return it }
        }
        return null
    }

    /** The title of the page the window shows; empty when it shows none. */
    private fun pageTitle(): String {
        var title = ""
        scenario.onActivity { activity -> title = webViewIn(activity.window.decorView)?.title.orEmpty() }
        return title
    }

    private fun awaitPage(title: String) {
        try {
            compose.waitUntil(15_000) { pageTitle() == title }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the page <$title> never showed; the window shows <${pageTitle()}>")
        }
    }

    /** Sends the tab in front to a page of the test site, and waits for it by its title. */
    private fun go(path: String, title: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url(path))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage(title)
    }

    /** What each tab of the strip says of itself beside its name, and its name. */
    private fun tabs(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { node ->
            val name = node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text }
            val said = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty()
            if (said.isEmpty()) name else "$name ($said)"
        }

    private fun awaitTabs(vararg expected: String) {
        try {
            compose.waitUntil(15_000) { tabs() == expected.toList() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the strip never was ${expected.toList()}; it is ${tabs()}")
        }
    }

    /** Each Group's chip in the strip: what it shows, and what it says of itself. */
    private fun groups(): List<String> =
        compose.onAllNodesWithTag("group").fetchSemanticsNodes().map { node ->
            val shown = node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text }
            "$shown: ${node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty()}"
        }

    private fun awaitGroups(vararg expected: String) {
        try {
            compose.waitUntil(15_000) { groups() == expected.toList() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the Groups never were ${expected.toList()}; they are ${groups()}")
        }
    }

    /** Holds something of the strip, and presses the item of its menu that has this tag: the first of them. */
    private fun fromTheMenuOf(tag: String, index: Int, item: String) {
        compose.onAllNodesWithTag(tag)[index].performScrollTo().performTouchInput { longClick() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag(item).fetchSemanticsNodes().isNotEmpty() }
        compose.onAllNodesWithTag(item)[0].performClick()
    }

    private fun pressTheChip() {
        compose.onAllNodesWithTag("group")[0].performScrollTo().performClick()
    }

    @Test
    fun aTabIsPutInANewGroupWhichIsShutOpenedEditedAndRemoved() {
        awaitPage("Home")
        awaitTabs("Home")
        awaitGroups()

        // "New Group…", from the tab's own menu: the page asks for a name. Made, the Group has
        // its chip in the row, open, and the tab is a Site: put in a Group, it is kept.
        fromTheMenuOf("tab", 0, "group-new")
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("group-name").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("group-name").performTextInput("Work")
        compose.onNodeWithTag("group-done").performScrollTo().performClick()
        awaitGroups("Work: Group, open")
        awaitTabs("Home (Site)")
        assertEquals("the page closed", 0, compose.onAllNodesWithTag("group-page").fetchSemanticsNodes().size)

        // A second tab, somewhere else: something to show while the Group is shut.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/whoami", "who=none")
        awaitTabs("Home (Site)", "who=none")
        val asked = site.requests.size

        // A press on the chip shuts the Group: its Site is out of the way, and the chip says
        // how many it holds. Pressed again it is open, and the Site is where it was. Neither
        // loads anything.
        pressTheChip()
        awaitGroups("Work (1): Group, shut, Sites: 1")
        awaitTabs("who=none")
        assertEquals("the tab in front is as it was", "who=none", pageTitle())
        pressTheChip()
        awaitGroups("Work: Group, open")
        awaitTabs("Home (Site)", "who=none")
        assertEquals("shutting and opening a Group loaded nothing", asked, site.requests.size)

        // The Group's own menu: edited, it has another name.
        fromTheMenuOf("group", 0, "group-edit")
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("group-name").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("group-name").performTextReplacement("Office")
        compose.onNodeWithTag("group-done").performScrollTo().performClick()
        awaitGroups("Office: Group, open")

        // The Site is taken out of its Group by "No Group": it is a Site still, and the Group,
        // shut, hides nothing and holds nothing.
        fromTheMenuOf("tab", 0, "group-none")
        pressTheChip()
        awaitGroups("Office (0): Group, shut, Sites: 0")
        awaitTabs("Home (Site)", "who=none")
        pressTheChip()
        awaitGroups("Office: Group, open")

        // Put back by the Group's name, and the Group shut: out of the way again, the other
        // tab being in front.
        fromTheMenuOf("tab", 0, "move-to-group")
        pressTheChip()
        awaitGroups("Office (1): Group, shut, Sites: 1")
        awaitTabs("who=none")

        // Removed, shut as it is: the chip is gone, and its Site stays, a Site in no Group.
        fromTheMenuOf("group", 0, "group-remove")
        awaitGroups()
        awaitTabs("Home (Site)", "who=none")
    }
}
