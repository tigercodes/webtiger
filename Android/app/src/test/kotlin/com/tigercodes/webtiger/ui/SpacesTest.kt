// Several Spaces (024 §6.2), drawn against a scripted core: the chip that names the Space the
// window shows, the list it opens, what a press and a new Space ask of the core, and what is
// said when the core will not make one.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SpaceCreated
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
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
class SpacesTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    /** What the scripted core lists, and why it will not make a Space, when it will not. */
    private var listed =
        listOf(
            SpaceSummary(space_id = 1, name = "", logins = 1),
            SpaceSummary(space_id = 2, name = "Work", logins = 2),
            SpaceSummary(space_id = 3, name = "Project", logins = 2),
        )
    private var refusal: String? = null
    private val core = ScriptedCore(::script)

    private fun script(request: Request): Response =
        when {
            request.query_spaces != null -> Response(spaces_page = SpacesPage(spaces = listed))
            request.create_space != null ->
                refusal?.let { Response(error = Error(code = it)) } ?: run {
                    val id = listed.size + 1L
                    listed = listed + SpaceSummary(space_id = id, name = request.create_space!!.name, logins = id.toInt())
                    Response(space_created = SpaceCreated(space_id = id))
                }
            else -> windowScript(request)
        }

    @After
    fun end() {
        scope.cancel()
    }

    /** A window that the core says shows the Space `shown`, called `name`. */
    private fun aWindow(shown: Long = 1, name: String = "") {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
        core.emit(snapshot().copy(space_id = shown, space_name = name), subscription = 3)
        compose.waitForIdle()
    }

    private fun lists(): Int = compose.onAllNodesWithTag("spaces").fetchSemanticsNodes().size

    /** What was asked of the core about Spaces, in order, in a few words each. */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.query_spaces?.let { "list" }
                ?: request.switch_space?.let { "show ${it.space_id} in ${it.window_id}" }
                ?: request.create_space?.let { "make '${it.name}' sharing ${it.share_logins_with}" }
        }

    private fun description(tag: String): String? =
        compose
            .onNodeWithTag(tag)
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.ContentDescription)
            ?.joinToString()

    @Test
    fun theChipNamesTheSpaceTheWindowShowsAndTheOneNobodyNamedHasTheShellsWord() {
        aWindow()
        assertEquals("Space: Home", description("space-chip"))
        assertEquals("nothing is asked until the list is opened", emptyList<String>(), asked())

        core.emit(snapshot().copy(space_id = 2, space_name = "Work"), subscription = 3)
        compose.waitForIdle()
        assertEquals("Space: Work", description("space-chip"))
    }

    @Test
    fun theChipOpensTheListWhichIsTheCoresAnswerWithTheOneShownMarked() {
        aWindow(shown = 2, name = "Work")
        assertEquals(0, lists())

        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()

        assertEquals(1, lists())
        assertEquals(listOf("list"), asked())
        // Each by its name, in the core's order; who shares logins says so; one is marked.
        compose.onNodeWithTag("space-1").assertTextEquals("Home")
        compose.onNodeWithTag("space-2").assertTextEquals("Work", "Shares logins with Project", "Shown")
        compose.onNodeWithTag("space-3").assertTextEquals("Project", "Shares logins with Work")
        compose.onAllNodesWithTag("space-shown", useUnmergedTree = true).assertCountEquals(1)

        // Done: the list goes, and nothing was asked for.
        compose.onNodeWithTag("spaces-done").performClick()
        compose.waitForIdle()
        assertEquals(0, lists())
        assertEquals(listOf("list"), asked())

        // Opened again, it is asked for again.
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "list"), asked())
    }

    @Test
    fun aSpaceThatIsPressedIsAskedOfTheCoreAndTheListGoes() {
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("space-3").performClick()
        compose.waitForIdle()

        assertEquals(listOf("list", "show 3 in 1"), asked())
        assertEquals(0, lists())
        // The chip says what the core says, and the core has not said anything yet.
        assertEquals("Space: Home", description("space-chip"))
    }

    @Test
    fun aSpaceIsMadeWithItsNameAsTypedAndLoginsOfItsOwnAndIsThenShown() {
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.waitForIdle()

        // Nothing is made of no name, nor of white space.
        compose.onNodeWithTag("space-make").assertIsNotEnabled()
        compose.onNodeWithTag("space-logins-own").assertIsSelected()
        compose.onNodeWithTag("space-logins-shared").assertIsNotSelected()
        compose.onNodeWithTag("space-name").performTextInput("  ")
        compose.onNodeWithTag("space-make").assertIsNotEnabled()

        compose.onNodeWithTag("space-name").performTextInput("Travel ")
        compose.onNodeWithTag("space-make").assertIsEnabled().performClick()
        compose.waitForIdle()

        // As typed: tidying a name is the core's.
        assertEquals(listOf("list", "make '  Travel ' sharing 0", "show 4 in 1"), asked())
        assertEquals("made, and the list is gone", 0, lists())
    }

    @Test
    fun aSpaceIsMadeToShareTheLoginsOfTheSpaceShownOrOfAnotherThatHasItsOwn() {
        // The window shows Project, whose logins are Work's: sharing them is sharing Work's.
        aWindow(shown = 3, name = "Project")
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.onNodeWithTag("space-name").performTextInput("Clients")
        compose.onNodeWithTag("space-logins-shared").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("space-logins-shared").assertIsSelected()
        compose.onNodeWithTag("space-logins-shared").assertTextEquals("The same as another Space", "Signed in wherever Work is. The two share history and downloads.")
        // The Spaces to choose among are those with logins of their own: not Project.
        compose.onNodeWithTag("space-share-with-2").assertIsSelected()
        compose.onNodeWithTag("space-share-with-1").assertIsNotSelected()
        compose.onAllNodesWithTag("space-share-with-3").assertCountEquals(0)

        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "make 'Clients' sharing 2", "show 4 in 1"), asked())

        // Another's: the app's own Space.
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.onNodeWithTag("space-name").performTextInput("Family")
        compose.onNodeWithTag("space-logins-shared").performClick()
        compose.onNodeWithTag("space-share-with-1").performScrollTo().performClick()
        compose.onNodeWithTag("space-logins-shared").assertTextEquals("The same as another Space", "Signed in wherever Home is. The two share history and downloads.")
        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals("make 'Family' sharing 1", asked().dropLast(1).last())
    }

    @Test
    fun aSpaceTheCoreWillNotMakeSaysWhyInWordsAndThePageStays() {
        refusal = "space.too_many"
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.onNodeWithTag("space-name").performTextInput("One more")
        compose.onAllNodesWithTag("space-refusal").assertCountEquals(0)

        compose.onNodeWithTag("space-make").performClick()
        compose.waitForIdle()

        assertEquals(listOf("list", "make 'One more' sharing 0"), asked())
        assertEquals("the page stays", 1, lists())
        compose.onNodeWithTag("space-refusal").assertTextEquals("There are as many Spaces as there can be.")

        // Back to the list, and the list opened anew, say nothing of it.
        compose.onNodeWithTag("space-back").performClick()
        compose.onNodeWithTag("spaces-done").performClick()
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("space-refusal").assertCountEquals(0)
    }

    @Test
    fun whereThereIsNoRoomForANameASpaceIsItsInitial() {
        assertEquals("H", spaceInitial("Home"))
        assertEquals("as a capital", "W", spaceInitial("work"))
        assertEquals("É", spaceInitial("élan"))
        // A picture is one letter, whatever it is made of.
        assertEquals("👩‍🚀", spaceInitial("👩‍🚀 Missions"))
        assertEquals("", spaceInitial(""))
    }

    /** At a phone's width the chip is the initial of the Space's name; at a tablet's, the name. Either way a screen reader is told the name. */
    @Test
    @Config(qualifiers = "w411dp-h800dp")
    fun inANarrowWindowTheChipIsTheSpacesInitial() {
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").assertTextEquals("W")
        assertEquals("Space: Work", description("space-chip"))
    }

    @Test
    @Config(qualifiers = "w800dp-h600dp")
    fun inAWideWindowTheChipIsTheSpacesName() {
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").assertTextEquals("Work")
        assertEquals("Space: Work", description("space-chip"))
    }

    @Test
    fun aRefusalIsSaidInWordsAndNeverByItsCode() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        val words = { code: String -> spaceRefusalWords(context.resources, context.packageName, code) }
        assertEquals("There are as many Spaces as there can be.", words("space.too_many"))
        assertEquals("A Space needs a name.", words("space.name_empty"))
        assertEquals("That Space isn’t there any more.", words("space.unknown"))
        // A code with no words of its own, a newer core's or the store's, and no code at all.
        for (code in listOf("space.random", "store.query", "space.failed", "", "Not a code")) {
            assertEquals(code, "The Space couldn’t be made.", words(code))
        }
    }
}
