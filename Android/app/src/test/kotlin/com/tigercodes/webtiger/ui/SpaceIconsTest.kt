// A Space's icon (050), drawn against a scripted core: the chip has the icon the window says,
// in the place of the initial and before the name; the list has it beside the name; the pages
// that make a Space and edit one offer the icons the core lists; and what is chosen is asked of
// the core, after the colour and before the time. The shell has no list of icons of its own.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
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

private const val HOUSE = "🏠"
private const val BRIEFCASE = "💼"
private const val HEART = "❤️"

@RunWith(RobolectricTestRunner::class)
class SpaceIconsTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    /** What the scripted core lists: the Space nobody named, with no icon, and "Work", with a briefcase. */
    private var listed =
        listOf(
            SpaceSummary(space_id = 1, name = "", logins = 1),
            SpaceSummary(space_id = 2, name = "Work", logins = 2, colour = "blue", icon = BRIEFCASE),
        )

    /** The icons the scripted core lists: three, in an order of its own. */
    private var icons = listOf(HOUSE, BRIEFCASE, HEART)
    private var iconRefusal: String? = null
    private val core = ScriptedCore(::script)

    private fun script(request: Request): Response =
        when {
            request.query_spaces != null -> Response(spaces_page = SpacesPage(spaces = listed, colours = listOf("red", "blue"), tidy_times = listOf(0, 43_200), icons = icons))
            request.icon_space != null ->
                iconRefusal?.let { Response(error = Error(code = it)) } ?: run {
                    val given = request.icon_space!!
                    listed = listed.map { if (it.space_id == given.space_id) it.copy(icon = given.icon) else it }
                    Response(ack = Ack())
                }
            request.create_space != null -> {
                val id = listed.size + 1L
                listed = listed + SpaceSummary(space_id = id, name = request.create_space!!.name, logins = id.toInt(), icon = request.create_space!!.icon)
                Response(space_created = SpaceCreated(space_id = id))
            }
            else -> windowScript(request)
        }

    @After
    fun end() {
        scope.cancel()
    }

    /** A window that the core says shows the Space "Work", with this icon. */
    private fun aWindow(icon: String = BRIEFCASE) {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
        core.emit(snapshot().copy(space_id = 2, space_name = "Work", space_icon = icon), subscription = 3)
        compose.waitForIdle()
    }

    private fun openList() {
        compose.onNodeWithTag("space-chip").performClick()
        compose.waitForIdle()
    }

    private fun edit() {
        openList()
        compose.onNodeWithTag("spaces-rename").performClick()
        compose.waitForIdle()
    }

    private fun save() {
        compose.onNodeWithTag("space-save").performScrollTo().performClick()
        compose.waitForIdle()
    }

    private fun lists(): Int = compose.onAllNodesWithTag("spaces").fetchSemanticsNodes().size

    /** An icon by its code points: a picture in the message of a failed test helps nobody. */
    private fun points(icon: String): String = icon.codePoints().toArray().joinToString("+") { it.toString(16) }

    /** What was asked of the core to make or change a Space, in order, in a few words each. */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.create_space?.let { "make '${it.name}' with ${points(it.icon)}" }
                ?: request.rename_space?.let { "name ${it.space_id} '${it.name}'" }
                ?: request.colour_space?.let { "colour ${it.space_id} '${it.colour}'" }
                ?: request.icon_space?.let { "icon ${it.space_id} ${points(it.icon)}" }
                ?: request.set_space_tidying?.let { "tidy ${it.space_id} after ${it.after_seconds}" }
        }

    /** What the chip draws, piece by piece: its children's words, which a screen reader is not read. */
    private fun chipDraws(): List<String> =
        compose
            .onAllNodesWithTag("space-icon-mark", useUnmergedTree = true)
            .fetchSemanticsNodes()
            .mapNotNull { it.config.getOrNull(SemanticsProperties.Text)?.joinToString("") { text -> text.text } }

    private fun description(tag: String): String? =
        compose
            .onNodeWithTag(tag)
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.ContentDescription)
            ?.joinToString()

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun inANarrowWindowTheChipIsTheSpacesIconInThePlaceOfItsInitial() {
        aWindow()
        assertEquals("the icon the window said", listOf(BRIEFCASE), chipDraws())
        compose.onAllNodesWithTag("space-chip").assertCountEquals(1)
        // It is said by the Space's name all the same: the icon is not its name.
        assertEquals("Space: Work", description("space-chip"))

        // Another icon: the chip draws it. And none: the initial again, and no icon.
        core.emit(snapshot().copy(space_id = 2, space_name = "Work", space_icon = HEART), subscription = 3)
        compose.waitForIdle()
        assertEquals(listOf(HEART), chipDraws())
        core.emit(snapshot().copy(space_id = 2, space_name = "Work"), subscription = 3)
        compose.waitForIdle()
        assertEquals(emptyList<String>(), chipDraws())
    }

    @Test
    @Config(qualifiers = "w900dp-h600dp")
    fun inAWideWindowTheChipHasTheIconBeforeTheName() {
        aWindow()
        assertEquals(listOf(BRIEFCASE), chipDraws())
        assertEquals("Space: Work", description("space-chip"))
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun theListHasASpacesIconBesideItsNameAndTheRowSaysOnlyTheName() {
        aWindow()
        openList()
        // Work's row has its icon; the Space with none keeps the icon's place, so that the
        // names stay in line; and neither is read out with the row.
        compose.onAllNodesWithTag("space-icon-mark", useUnmergedTree = true).assertCountEquals(2) // the chip's and the row's
        compose.onAllNodesWithTag("space-no-icon-mark", useUnmergedTree = true).assertCountEquals(1)
        compose.onNodeWithTag("space-2").assertTextEquals("Work", "Shown")
        compose.onNodeWithTag("space-1").assertTextEquals("Home")
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun whereNoSpaceHasAnIconTheListKeepsNoPlaceForOne() {
        listed = listed.map { it.copy(icon = "") }
        aWindow(icon = "")
        openList()
        compose.onAllNodesWithTag("space-icon-mark", useUnmergedTree = true).assertCountEquals(0)
        compose.onAllNodesWithTag("space-no-icon-mark", useUnmergedTree = true).assertCountEquals(0)
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun thePageThatEditsASpaceOffersTheCoresIconsWithTheSpacesOwnChosenAndAsksForAnother() {
        aWindow()
        edit()
        // None, and the core's three in its order: the second is the Space's.
        compose.onNodeWithTag("space-icon").performScrollTo().assertTextEquals("Icon")
        compose.onNodeWithTag("space-icon-none").performScrollTo().assertIsNotSelected()
        assertEquals("No icon", description("space-icon-none"))
        compose.onNodeWithTag("space-icon-1").assertTextEquals(HOUSE)
        compose.onNodeWithTag("space-icon-2").assertTextEquals(BRIEFCASE)
        compose.onNodeWithTag("space-icon-3").assertTextEquals(HEART)
        compose.onNodeWithTag("space-icon-2").assertIsSelected()
        compose.onNodeWithTag("space-icon-1").assertIsNotSelected()
        compose.onAllNodesWithTag("space-icon-4").assertCountEquals(0)

        // Another icon, and the rest as it was: only the icon is asked for, and the list goes.
        compose.onNodeWithTag("space-icon-3").performScrollTo().performClick()
        compose.onNodeWithTag("space-icon-3").assertIsSelected()
        compose.onNodeWithTag("space-icon-2").assertIsNotSelected()
        save()
        assertEquals(listOf("icon 2 2764+fe0f"), asked())
        assertEquals("saved, and the list is gone", 0, lists())

        // The page says the icon the core now lists. With a colour and a time changed too:
        // the colour, then the icon, then the time; and none is an icon too.
        edit()
        compose.onNodeWithTag("space-icon-3").performScrollTo().assertIsSelected()
        compose.onNodeWithTag("space-colour-red").performScrollTo().performClick()
        compose.onNodeWithTag("space-icon-none").performScrollTo().performClick()
        compose.onNodeWithTag("space-icon-none").assertIsSelected()
        compose.onNodeWithTag("space-tidy-43200").performScrollTo().performClick()
        save()
        assertEquals(listOf("colour 2 'red'", "icon 2 ", "tidy 2 after 43200"), asked().drop(1))

        // The icon it has, pressed again: nothing is asked.
        edit()
        compose.onNodeWithTag("space-icon-none").performScrollTo().performClick()
        val before = asked().size
        save()
        assertEquals(before, asked().size)
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun aSpaceIsMadeWithTheIconChosenOrWithNone() {
        aWindow()
        openList()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.waitForIdle()
        // None is the one chosen until another is: a Space is given no icon it was not asked for.
        compose.onNodeWithTag("space-icon-none").performScrollTo().assertIsSelected()
        compose.onNodeWithTag("space-name").performScrollTo().performTextInput("Travel")
        compose.onNodeWithTag("space-icon-1").performScrollTo().performClick()
        compose.onNodeWithTag("space-icon-1").assertIsSelected()
        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("make 'Travel' with 1f3e0"), asked())

        openList()
        compose.onNodeWithTag("spaces-new").performClick()
        compose.onNodeWithTag("space-name").performScrollTo().performTextInput("Plain")
        compose.onNodeWithTag("space-make").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals("make 'Plain' with ", asked().last())
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun anIconTheCoreWillNotTakeSaysWhyInWordsAndThePageStays() {
        iconRefusal = "space.icon_unknown"
        aWindow()
        edit()
        compose.onNodeWithTag("space-icon-1").performScrollTo().performClick()
        compose.onNodeWithTag("space-tidy-43200").performScrollTo().performClick()
        save()

        assertEquals("the time was not asked for", listOf("icon 2 1f3e0"), asked())
        assertEquals("the page is still there", 1, lists())
        compose.onNodeWithTag("space-refusal").performScrollTo().assertTextEquals("That icon isn’t one a Space can have.")
    }

    @Test
    @Config(qualifiers = "w411dp-h900dp")
    fun aCoreThatListsNoIconsHasNoChoiceOfThem() {
        icons = emptyList()
        aWindow()
        edit()
        compose.onAllNodesWithTag("space-icon").assertCountEquals(0)
        compose.onAllNodesWithTag("space-icon-none").assertCountEquals(0)
    }
}
