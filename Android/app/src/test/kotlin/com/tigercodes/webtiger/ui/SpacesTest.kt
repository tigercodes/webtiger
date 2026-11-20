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
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SpaceCreated
import com.tigercodes.webtiger.core.v1.SpaceSummary
import com.tigercodes.webtiger.core.v1.SpacesPage
import com.tigercodes.webtiger.design.Tokens
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
            SpaceSummary(space_id = 2, name = "Work", logins = 2, colour = "blue"),
            SpaceSummary(space_id = 3, name = "Project", logins = 2),
        )

    /** The colours the scripted core lists: three this shell has paints for, and one it has not. */
    private var colours = listOf("red", "orange", "blue", "mauve")
    private var colourRefusal: String? = null
    private var refusal: String? = null
    private var renameRefusal: String? = null
    private val core = ScriptedCore(::script)

    private fun script(request: Request): Response =
        when {
            request.query_spaces != null -> Response(spaces_page = SpacesPage(spaces = listed, colours = colours))
            request.colour_space != null ->
                colourRefusal?.let { Response(error = Error(code = it)) } ?: run {
                    val coloured = request.colour_space!!
                    listed = listed.map { if (it.space_id == coloured.space_id) it.copy(colour = coloured.colour) else it }
                    Response(ack = Ack())
                }
            request.create_space != null ->
                refusal?.let { Response(error = Error(code = it)) } ?: run {
                    val id = listed.size + 1L
                    listed = listed + SpaceSummary(space_id = id, name = request.create_space!!.name, logins = id.toInt())
                    Response(space_created = SpaceCreated(space_id = id))
                }
            request.rename_space != null ->
                renameRefusal?.let { Response(error = Error(code = it)) } ?: run {
                    val named = request.rename_space!!
                    listed = listed.map { if (it.space_id == named.space_id) it.copy(name = named.name) else it }
                    Response(ack = Ack())
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
                ?: request.create_space?.let { "make '${it.name}' sharing ${it.share_logins_with}" + if (it.colour.isEmpty()) "" else " in ${it.colour}" }
                ?: request.rename_space?.let { "name ${it.space_id} '${it.name}'" }
                ?: request.colour_space?.let { "colour ${it.space_id} '${it.colour}'" }
        }

    /** What the field for a Space's name holds. */
    private fun typed(): String =
        compose
            .onNodeWithTag("space-name")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.EditableText)
            ?.text
            .orEmpty()

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
        compose.onNodeWithTag("space-make").assertIsEnabled().performScrollTo().performClick()
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

        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()

        assertEquals(listOf("list", "make 'One more' sharing 0"), asked())
        assertEquals("the page stays", 1, lists())
        compose.onNodeWithTag("space-refusal").assertTextEquals("There are as many Spaces as there can be.")

        // Back to the list, and the list opened anew, say nothing of it.
        compose.onNodeWithTag("space-back").performScrollTo().performClick()
        compose.onNodeWithTag("spaces-done").performClick()
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("space-refusal").assertCountEquals(0)
    }

    @Test
    fun theSpaceShownIsNamedWithTheNameAsTypedAndTheListGoes() {
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").assertIsEnabled().performClick()
        compose.waitForIdle()

        // The field holds the name the Space has.
        assertEquals("Work", typed())
        compose.onNodeWithTag("space-save").assertIsEnabled()
        compose.onNodeWithTag("space-name").performTextReplacement("  The office ")
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()

        // The Space the window shows, and the name as typed: tidying it is the core's.
        assertEquals(listOf("list", "name 2 '  The office '"), asked())
        assertEquals("named, and the list is gone", 0, lists())
        // The chip says what the core says, and the core has not said anything yet.
        assertEquals("Space: Work", description("space-chip"))
    }

    @Test
    fun theSpaceNobodyNamedStartsWithAnEmptyFieldAndASpaceWithANameIsNotGivenNone() {
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.waitForIdle()

        // "Home" is the shell's word for it, not a name it was given: the field is empty. It
        // may keep no name, and white space is no name: saved so, nothing is asked.
        assertEquals("", typed())
        compose.onNodeWithTag("space-name").performTextInput("   ")
        compose.onNodeWithTag("space-save").assertIsEnabled().performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list"), asked())
        assertEquals(0, lists())

        // A Space that has a name is not given none.
        core.emit(snapshot().copy(space_id = 2, space_name = "Work"), subscription = 3)
        compose.waitForIdle()
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.onNodeWithTag("space-name").performTextReplacement("  ")
        compose.onNodeWithTag("space-save").assertIsNotEnabled()
        compose.onNodeWithTag("space-name").performTextReplacement("Home base")
        compose.onNodeWithTag("space-save").assertIsEnabled()

        // Back: nothing was asked for, and the list is there again.
        compose.onNodeWithTag("space-back").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "list"), asked())
        compose.onAllNodesWithTag("spaces-rename").assertCountEquals(1)
    }

    @Test
    fun aNameTheCoreWillNotTakeSaysWhyInWordsAndThePageStays() {
        renameRefusal = "space.unknown"
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.onNodeWithTag("space-name").performTextReplacement("Office")
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()

        assertEquals(listOf("list", "name 2 'Office'"), asked())
        assertEquals("the page stays", 1, lists())
        compose.onNodeWithTag("space-refusal").assertTextEquals("That Space isn’t there any more.")

        // A code with no words of its own is said as "it could not be renamed", not "made".
        renameRefusal = "store.query"
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("space-refusal").assertTextEquals("The Space couldn’t be changed.")
    }

    @Test
    fun untilTheCoreHasListedTheSpaceShownThereIsNoneToName() {
        // A core that lists nothing of the Space the window says it shows.
        aWindow(shown = 9, name = "Elsewhere")
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").assertIsNotEnabled()
    }

    // A Space's colour (025)

    private fun marks(tag: String): Int = compose.onAllNodesWithTag(tag, useUnmergedTree = true).fetchSemanticsNodes().size

    @Test
    fun theChipAndTheListHaveTheMarkOfASpacesColourAndASpaceWithNoneHasNone() {
        aWindow()
        assertEquals("the app's own Space has no colour", 0, marks("space-mark-blue"))

        core.emit(snapshot().copy(space_id = 2, space_name = "Work", space_colour = "blue"), subscription = 3)
        compose.waitForIdle()
        assertEquals("the chip", 1, marks("space-mark-blue"))
        // Told by its name all the same: the mark is no part of what a screen reader says.
        assertEquals("Space: Work", description("space-chip"))

        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        assertEquals("the chip, and Work's row", 2, marks("space-mark-blue"))
        // A colour this shell has no paint for is shown as none, and is no error.
        core.emit(snapshot().copy(space_id = 2, space_name = "Work", space_colour = "mauve"), subscription = 3)
        compose.waitForIdle()
        assertEquals("only the row's now", 1, marks("space-mark-blue"))
        assertEquals(0, marks("space-mark-mauve"))
    }

    @Test
    fun aNewSpaceIsMadeWithTheColourChosenOrWithNoneForTheCoreToChoose() {
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.waitForIdle()

        // The colours the core lists that this shell can paint, each by its word; none chosen.
        compose.onNodeWithTag("space-colour").assertTextEquals("Colour")
        for (colour in listOf("red", "orange", "blue")) compose.onNodeWithTag("space-colour-$colour").assertIsNotSelected()
        compose.onAllNodesWithTag("space-colour-mauve").assertCountEquals(0)
        compose.onAllNodesWithTag("space-colour-none").assertCountEquals(0)
        assertEquals("Orange", description("space-colour-orange"))

        compose.onNodeWithTag("space-name").performTextInput("Travel")
        compose.onNodeWithTag("space-colour-orange").performScrollTo().performClick()
        compose.onNodeWithTag("space-colour-orange").assertIsSelected()
        // Said in words too: not by the colour alone.
        compose.onNodeWithTag("space-colour").assertTextEquals("Colour: Orange")
        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "make 'Travel' sharing 0 in orange", "show 4 in 1"), asked())

        // None chosen: the core chooses.
        compose.onNodeWithTag("space-chip").performClick()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.onNodeWithTag("space-name").performTextInput("Plain")
        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals("make 'Plain' sharing 0", asked().dropLast(1).last())
    }

    @Test
    fun editingASpaceAsksForWhatChangedTheNameAndTheColour() {
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.waitForIdle()

        // The colour it has is the one chosen, and no colour is a choice too.
        compose.onNodeWithTag("space-colour-blue").assertIsSelected()
        compose.onNodeWithTag("space-colour").assertTextEquals("Colour: Blue")
        compose.onNodeWithTag("space-colour-none").assertIsNotSelected()
        assertEquals("None", description("space-colour-none"))

        // Another colour, and the name as it was: only the colour is asked for.
        compose.onNodeWithTag("space-colour-red").performClick()
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "colour 2 'red'"), asked())
        assertEquals("saved, and the list is gone", 0, lists())

        // Both, the name first; and none.
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.onNodeWithTag("space-name").performTextReplacement("Office")
        compose.onNodeWithTag("space-colour-none").performClick()
        compose.onNodeWithTag("space-colour").assertTextEquals("Colour")
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("name 2 'Office'", "colour 2 ''"), asked().takeLast(2))

        // Nothing changed: nothing is asked, and the list goes.
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        val before = asked().size
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals("only the list was asked for", before, asked().size)
        assertEquals(0, lists())
    }

    @Test
    fun theSpaceNobodyNamedIsGivenAColourWithoutAName() {
        aWindow()
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.waitForIdle()

        // It has no name, and may keep none: the colour alone is saved.
        assertEquals("", typed())
        compose.onNodeWithTag("space-colour-none").assertIsSelected()
        compose.onNodeWithTag("space-colour-orange").performClick()
        compose.onNodeWithTag("space-save").assertIsEnabled().performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("list", "colour 1 'orange'"), asked())
    }

    @Test
    fun aNameTheCoreWillNotTakeLeavesTheColourUnaskedAndAColourItWillNotTakeSaysWhy() {
        renameRefusal = "space.name_empty"
        aWindow(shown = 2, name = "Work")
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.onNodeWithTag("space-name").performTextReplacement("Office")
        compose.onNodeWithTag("space-colour-red").performClick()
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals("the colour was not asked for", listOf("list", "name 2 'Office'"), asked())
        assertEquals(1, lists())

        renameRefusal = null
        colourRefusal = "space.colour_unknown"
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("space-refusal").assertTextEquals("That colour isn’t one a Space can have.")
        assertEquals(1, lists())
    }

    @Test
    fun everyColourHasAWordAndAPaintAndAnotherNameHasNeither() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        val word = { name: String -> spaceColourWord(context.resources, context.packageName, name) }
        val names = Tokens.SpaceColour.named.map { it.first }
        assertEquals(listOf("red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "grey"), names)
        assertEquals(listOf("Red", "Orange", "Yellow", "Green", "Teal", "Blue", "Purple", "Pink", "Grey"), names.map(word))
        // None, and a colour a newer core might say: no paint, and never its own text.
        for (other in listOf("", "mauve", "Blue", "none", "chosen", "automatic", "space.colour")) {
            assertEquals(other, null, Tokens.SpaceColour.paint(other))
            assertEquals(other, "", word(other))
        }
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
