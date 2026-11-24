// A Space's time (049), drawn against a scripted core: the page that edits a Space has the
// times the core lists, with the Space's own chosen; what is saved is asked of the core, after
// the name and the colour, and only when it is another; and what the core will not take is said
// in words. And the window's Library has no Archive where the window is a private one.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SpaceSummary
import com.tigercodes.webtiger.core.v1.SpacesPage
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

@RunWith(RobolectricTestRunner::class)
class SpaceTidyingTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    /** What the scripted core lists: the Space nobody named, with no time, and "Work", tidied after a day. */
    private var listed =
        listOf(
            SpaceSummary(space_id = 1, name = "", logins = 1),
            SpaceSummary(space_id = 2, name = "Work", logins = 2, colour = "blue", tidy_after_seconds = 86_400),
        )

    /** The times the scripted core lists: the five this shell has words for, and an hour, for which it has none. */
    private var times = listOf(0, 3_600, 43_200, 86_400, 604_800, 2_592_000)
    private var timeRefusal: String? = null
    private val core = ScriptedCore(::script)

    private fun script(request: Request): Response =
        when {
            request.query_spaces != null -> Response(spaces_page = SpacesPage(spaces = listed, colours = listOf("red", "blue"), tidy_times = times))
            request.set_space_tidying != null ->
                timeRefusal?.let { Response(error = Error(code = it)) } ?: run {
                    val set = request.set_space_tidying!!
                    listed = listed.map { if (it.space_id == set.space_id) it.copy(tidy_after_seconds = set.after_seconds) else it }
                    Response(ack = Ack())
                }
            else -> windowScript(request)
        }

    @After
    fun end() {
        scope.cancel()
    }

    /** A window that the core says shows the Space "Work", with the page that edits it open. */
    private fun editingWork() {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
        core.emit(snapshot().copy(space_id = 2, space_name = "Work"), subscription = 3)
        compose.waitForIdle()
        edit()
    }

    private fun edit() {
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.waitForIdle()
    }

    private fun save() {
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
    }

    private fun lists(): Int = compose.onAllNodesWithTag("spaces").fetchSemanticsNodes().size

    /** What was asked of the core to change a Space, in order, in a few words each. */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.rename_space?.let { "name ${it.space_id} '${it.name}'" }
                ?: request.colour_space?.let { "colour ${it.space_id} '${it.colour}'" }
                ?: request.set_space_tidying?.let { "tidy ${it.space_id} after ${it.after_seconds}" }
        }

    @Test
    fun thePageHasTheTimesTheCoreListsThatHaveWordsWithTheSpacesOwnChosen() {
        editingWork()
        compose.onNodeWithTag("space-tidy").performScrollTo().assertTextEquals("Tidy tabs away")
        for ((time, words) in listOf(0 to "Never", 43_200 to "After 12 hours", 86_400 to "After a day", 604_800 to "After a week", 2_592_000 to "After 30 days")) {
            compose.onNodeWithTag("space-tidy-$time").performScrollTo().assertTextEquals(words)
        }
        // The one the Space has is the one chosen, and only it.
        compose.onNodeWithTag("space-tidy-86400").assertIsSelected()
        compose.onNodeWithTag("space-tidy-0").assertIsNotSelected()
        // A time this build has no words for is not offered, and never shown as a number.
        compose.onAllNodesWithTag("space-tidy-3600").assertCountEquals(0)
        compose.onNodeWithText("3600", substring = true).assertDoesNotExist()
    }

    @Test
    fun aTimeChosenIsAskedOfTheCoreAfterTheNameAndTheColourAndOnlyWhenItIsAnother() {
        editingWork()

        // Another time, and the rest as it was: only the time is asked for, and the list goes.
        compose.onNodeWithTag("space-tidy-43200").performScrollTo().performClick()
        compose.onNodeWithTag("space-tidy-43200").assertIsSelected()
        compose.onNodeWithTag("space-tidy-86400").assertIsNotSelected()
        save()
        assertEquals(listOf("tidy 2 after 43200"), asked())
        assertEquals("saved, and the list is gone", 0, lists())

        // The page says the time the core now lists. All three changed: the name first, then
        // the colour, then the time; and never is a time too.
        edit()
        compose.onNodeWithTag("space-tidy-43200").performScrollTo().assertIsSelected()
        compose.onNodeWithTag("space-name").performScrollTo().performTextReplacement("Office")
        compose.onNodeWithTag("space-colour-red").performScrollTo().performClick()
        compose.onNodeWithTag("space-tidy-0").performScrollTo().performClick()
        save()
        assertEquals(listOf("name 2 'Office'", "colour 2 'red'", "tidy 2 after 0"), asked().drop(1))

        // The time it has, pressed again: nothing is asked.
        edit()
        compose.onNodeWithTag("space-tidy-0").performScrollTo().performClick()
        val before = asked().size
        save()
        assertEquals(before, asked().size)
        assertEquals(0, lists())
    }

    @Test
    fun aTimeTheCoreWillNotTakeSaysWhyInWordsAndThePageStays() {
        timeRefusal = "tidy.time_unknown"
        editingWork()
        compose.onNodeWithTag("space-tidy-604800").performScrollTo().performClick()
        save()

        assertEquals(listOf("tidy 2 after 604800"), asked())
        assertEquals("the page is still there", 1, lists())
        compose.onNodeWithTag("space-refusal").performScrollTo().assertTextEquals("That isn’t a time tabs can be tidied after.")

        // Taken the next time, the list goes.
        timeRefusal = null
        save()
        assertEquals(0, lists())
    }

    @Test
    fun aCoreThatListsNoTimesHasNoChoiceOfThem() {
        times = emptyList()
        editingWork()
        compose.onAllNodesWithTag("space-tidy").assertCountEquals(0)
        compose.onAllNodesWithTag("space-tidy-0").assertCountEquals(0)
        // The page is as it was before there were times: saved with nothing changed, nothing is asked.
        save()
        assertEquals(emptyList<String>(), asked())
    }

    @Test
    fun everyTimeTheCoreListsHasWordsAndAnotherNumberHasNone() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        // The times the real core lists (`ThroughTheCoreTest` holds it to these).
        assertEquals(
            listOf("Never", "After 12 hours", "After a day", "After a week", "After 30 days"),
            listOf(0, 43_200, 86_400, 604_800, 2_592_000).map { tidyTimeWords(context.resources, context.packageName, it) },
        )
        for (other in listOf(1, 3_600, -1, 86_401)) {
            assertEquals("$other", "", tidyTimeWords(context.resources, context.packageName, other))
        }
    }

    @Test
    fun aPrivateWindowsLibraryHasNoArchive() {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session, asPrivate = true)) } }
        compose.waitForIdle()
        compose.onNodeWithContentDescription("History and Downloads").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("library-history").assertExists()
        compose.onAllNodesWithTag("library-archive").assertCountEquals(0)
        assertEquals("and its Archive is never asked for", 0, core.requests.count { it.query_archive != null })
    }
}
