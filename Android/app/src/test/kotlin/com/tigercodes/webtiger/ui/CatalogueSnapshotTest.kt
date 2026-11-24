// The design catalogue as pictures (004 §4): the token sections and each state of the window,
// in the light and the dark theme, drawn on the JVM and compared with the pictures checked in.
package com.tigercodes.webtiger.ui

import androidx.activity.ComponentActivity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.design.TokenColor
import com.tigercodes.webtiger.design.Tokens
import com.tigercodes.webtiger.design.WebTigerTheme
import java.io.File
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

private const val LIGHT = "w411dp-h320dp-notnight-xhdpi"
private const val DARK = "w411dp-h320dp-night-xhdpi"

// Tall enough for the Downloads list's five downloads, each in a different state, and what follows them.
private const val LIST_LIGHT = "w411dp-h560dp-notnight-xhdpi"
private const val LIST_DARK = "w411dp-h560dp-night-xhdpi"

/** Tall enough for every tab of what is awake (056), and the counts under them. */
private const val AWAKE_LIGHT = "w411dp-h760dp-notnight-xhdpi"
private const val AWAKE_DARK = "w411dp-h760dp-night-xhdpi"
// Tall enough for every token: the colours a Space can have are nine of them (025).
private const val TALL_LIGHT = "w411dp-h1300dp-notnight-xhdpi"
private const val TALL_DARK = "w411dp-h1300dp-night-xhdpi"

/** The states of the window whose bar is taller than the window's usual picture: drawn at a list's height, so that all of it is seen. */
private val TALL_STATES =
    setOf(
        CatalogueState.PROMPT_SIGN_IN_AGAIN,
        CatalogueState.SEARCH_QUESTION,
        CatalogueState.SEARCH_CHANGE,
        // A menu's choices, all of them: more than a window's picture has room for.
        CatalogueState.LINK_MENU,
        CatalogueState.PICTURE_MENU,
        CatalogueState.PICTURE_LINK_MENU,
    )

/**
 * The states of the window that are a page of words: a failed page with the way to go on past a
 * certificate under it, and with the warning (026). Drawn as tall as a page, so that the button
 * under the warning is in the picture.
 */
private val PAGE_STATES = setOf(CatalogueState.FAILED_CERTIFICATE, CatalogueState.FAILED_CERTIFICATE_WARNING)

/** How tall a state's picture is. */
private enum class Height { USUAL, TALL, PAGE }

private fun heightOf(state: CatalogueState): Height =
    when (state) {
        in PAGE_STATES -> Height.PAGE
        in TALL_STATES -> Height.TALL
        else -> Height.USUAL
    }

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class CatalogueSnapshotTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    /** Draws what the activity shows, once nothing is left to do. */
    private fun verify(name: String) {
        compose.waitForIdle()
        Snapshots.verify(Snapshots.picture(compose.activity.window.decorView), name)
    }

    private fun show(content: @Composable () -> Unit) {
        compose.setContent { WebTigerTheme { Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background, content = content) } }
    }

    private fun tokens(theme: String) {
        show { CatalogueTokens(Modifier.padding(Tokens.Space.l)) }
        verify("tokens-$theme")
    }

    private fun windows(theme: String, height: Height = Height.USUAL) {
        var state by mutableStateOf(CatalogueState.EMPTY)
        show { key(state) { CatalogueWindow(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueState.entries.filter { heightOf(it) == height }) {
            state = next
            verify("window-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun historyLists(theme: String) {
        var state by mutableStateOf(CatalogueHistory.LIST)
        show { key(state) { CatalogueHistoryList(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueHistory.entries) {
            state = next
            verify("history-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun downloadsLists(theme: String) {
        var state by mutableStateOf(CatalogueDownloads.LIST)
        show { key(state) { CatalogueDownloadsList(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueDownloads.entries) {
            state = next
            verify("downloads-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun permissionsLists(theme: String) {
        var state by mutableStateOf(CataloguePermissions.LIST)
        show { key(state) { CataloguePermissionsList(state, Modifier.fillMaxSize()) } }
        for (next in CataloguePermissions.entries) {
            state = next
            verify("permissions-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun archiveLists(theme: String) {
        var state by mutableStateOf(CatalogueArchive.LIST)
        show { key(state) { CatalogueArchiveList(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueArchive.entries) {
            state = next
            verify("archive-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun awakeLists(theme: String) {
        var state by mutableStateOf(CatalogueAwake.LIST)
        show { key(state) { CatalogueAwakeList(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueAwake.entries) {
            state = next
            verify("awake-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun spacesLists(theme: String) {
        var state by mutableStateOf(CatalogueSpaces.LIST)
        show { key(state) { CatalogueSpacesList(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueSpaces.entries) {
            state = next
            verify("spaces-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun groupPages(theme: String) {
        var state by mutableStateOf(CatalogueGroups.NEW)
        show { key(state) { CatalogueGroupPage(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueGroups.entries) {
            state = next
            verify("group-${next.name.lowercase().replace('_', '-')}-$theme")
        }
    }

    private fun startFailed(theme: String) {
        show { StartFailed("store.open") }
        verify("start-failed-$theme")
    }

    @Test @Config(qualifiers = TALL_LIGHT)
    fun theTokenSectionsInTheLightTheme() = tokens("light")

    @Test @Config(qualifiers = TALL_DARK)
    fun theTokenSectionsInTheDarkTheme() = tokens("dark")

    @Test @Config(qualifiers = LIGHT)
    fun everyStateOfTheWindowInTheLightTheme() = windows("light")

    @Test @Config(qualifiers = DARK)
    fun everyStateOfTheWindowInTheDarkTheme() = windows("dark")

    @Test @Config(qualifiers = LIST_LIGHT)
    fun theWindowsTallStatesInTheLightTheme() = windows("light", Height.TALL)

    @Test @Config(qualifiers = LIST_DARK)
    fun theWindowsTallStatesInTheDarkTheme() = windows("dark", Height.TALL)

    @Test @Config(qualifiers = "w411dp-h820dp-notnight-xhdpi")
    fun theWindowsPagesOfWordsInTheLightTheme() = windows("light", Height.PAGE)

    @Test @Config(qualifiers = "w411dp-h820dp-night-xhdpi")
    fun theWindowsPagesOfWordsInTheDarkTheme() = windows("dark", Height.PAGE)

    @Test @Config(qualifiers = LIGHT)
    fun everyStateOfTheHistoryListInTheLightTheme() = historyLists("light")

    @Test @Config(qualifiers = DARK)
    fun everyStateOfTheHistoryListInTheDarkTheme() = historyLists("dark")

    @Test @Config(qualifiers = LIST_LIGHT)
    fun everyStateOfTheDownloadsListInTheLightTheme() = downloadsLists("light")

    @Test @Config(qualifiers = LIST_DARK)
    fun everyStateOfTheDownloadsListInTheDarkTheme() = downloadsLists("dark")

    @Test @Config(qualifiers = LIGHT)
    fun everyStateOfThePermissionsListInTheLightTheme() = permissionsLists("light")

    @Test @Config(qualifiers = DARK)
    fun everyStateOfThePermissionsListInTheDarkTheme() = permissionsLists("dark")

    /**
     * A window as wide as a tablet's: the chip has room for the Space's name (024), and the
     * strip for a row with Groups (041), of which a phone's shows the beginning.
     */
    private fun wideWindow(theme: String) {
        var state by mutableStateOf(CatalogueState.TABS)
        show { key(state) { CatalogueWindow(state, Modifier.fillMaxSize()) } }
        for (next in listOf(CatalogueState.TABS, CatalogueState.GROUPS)) {
            state = next
            verify("window-${next.name.lowercase()}-wide-$theme")
        }
    }

    @Test @Config(qualifiers = "w720dp-h320dp-notnight-xhdpi")
    fun aWideWindowInTheLightTheme() = wideWindow("light")

    @Test @Config(qualifiers = "w720dp-h320dp-night-xhdpi")
    fun aWideWindowInTheDarkTheme() = wideWindow("dark")

    /** A window wide enough for a sidebar, with its tabs down the side (043): tall enough for every row. */
    private fun sidebar(theme: String) {
        show { CatalogueWindow(CatalogueState.SIDEBAR, Modifier.fillMaxSize()) }
        verify("window-sidebar-wide-$theme")
    }

    @Test @Config(qualifiers = "w720dp-h560dp-notnight-xhdpi")
    fun aWindowWithItsTabsAtTheSideInTheLightTheme() = sidebar("light")

    @Test @Config(qualifiers = "w720dp-h560dp-night-xhdpi")
    fun aWindowWithItsTabsAtTheSideInTheDarkTheme() = sidebar("dark")

    // The Archive (049), under the Library's switch (057): five names, the row moved to show its own.
    @Test @Config(qualifiers = LIST_LIGHT)
    fun everyStateOfTheArchiveInTheLightTheme() = archiveLists("light")

    @Test @Config(qualifiers = LIST_DARK)
    fun everyStateOfTheArchiveInTheDarkTheme() = archiveLists("dark")

    // What is awake (056): the last of the switch's names.
    @Test @Config(qualifiers = AWAKE_LIGHT)
    fun everyStateOfWhatIsAwakeInTheLightTheme() = awakeLists("light")

    @Test @Config(qualifiers = AWAKE_DARK)
    fun everyStateOfWhatIsAwakeInTheDarkTheme() = awakeLists("dark")

    // Taller than a list's picture: the page that makes a Space has its name, its logins, the
    // Spaces to share them with, its colour, its icon and what was refused; and the page that
    // edits one has the times its tabs are tidied after, under its icon (049, 050).
    @Test @Config(qualifiers = "w411dp-h1100dp-notnight-xhdpi")
    fun everyStateOfTheSpacesInTheLightTheme() = spacesLists("light")

    @Test @Config(qualifiers = "w411dp-h1100dp-night-xhdpi")
    fun everyStateOfTheSpacesInTheDarkTheme() = spacesLists("dark")

    // As tall as the Spaces' pages: the colours take two rows, and the buttons are under what
    // was refused.
    @Test @Config(qualifiers = "w411dp-h820dp-notnight-xhdpi")
    fun everyStateOfTheGroupPageInTheLightTheme() = groupPages("light")

    @Test @Config(qualifiers = "w411dp-h820dp-night-xhdpi")
    fun everyStateOfTheGroupPageInTheDarkTheme() = groupPages("dark")

    /** The phone's switcher (047) over a phone's window: as tall as a page, for every row. */
    private fun switcher(theme: String) {
        show { CatalogueSwitcher(Modifier.fillMaxSize()) }
        verify("switcher-$theme")
    }

    @Test @Config(qualifiers = "w411dp-h820dp-notnight-xhdpi")
    fun thePhonesSwitcherInTheLightTheme() = switcher("light")

    @Test @Config(qualifiers = "w411dp-h820dp-night-xhdpi")
    fun thePhonesSwitcherInTheDarkTheme() = switcher("dark")

    @Test @Config(qualifiers = LIGHT)
    fun startFailedInTheLightTheme() = startFailed("light")

    @Test @Config(qualifiers = DARK)
    fun startFailedInTheDarkTheme() = startFailed("dark")

    /**
     * The words of a failure at twice the text size, on a screen as narrow as any: they wrap and
     * scroll, and none runs off the screen. (Apple's audit found the same page's words cut short
     * on the iPhone Duo's simulator; here they ran off both ends of the space they had.)
     */
    @Test @Config(qualifiers = "w320dp-h480dp-notnight-xhdpi", fontScale = 2f)
    fun aFailedPageAtTwiceTheTextSize() {
        show { FailedPage(code = "net.dns", onRetry = {}) }
        verify("failed-largest-text-light")
    }

    // The catalogue's lists, and the comparison itself

    @Test
    fun everyTokenIsInTheCatalogue() {
        // Design/tokens.json, found from where the pictures are kept: Android/app/src/test/snapshots.
        var root = File(checkNotNull(System.getProperty("webtiger.snapshots")))
        repeat(5) { root = checkNotNull(root.parentFile) }
        val file = JSONObject(File(root, "Design/tokens.json").readText())
        assertEquals("every colour, by its name", file.getJSONObject("color").keys().asSequence().toSet(), Tokens.Catalogue.colors.map { it.first }.toSet())
        for ((group, listed) in listOf("space" to Tokens.Catalogue.space, "radius" to Tokens.Catalogue.radius, "size" to Tokens.Catalogue.size)) {
            val written = file.getJSONObject(group)
            assertEquals(group, written.keys().asSequence().associateWith { written.getDouble(it) }, listed.associate { it.first to it.second.value.toDouble() })
        }
    }

    @Test @Config(qualifiers = "w200dp-h100dp-notnight-mdpi")
    fun theComparisonSeesAChange() {
        var colour by mutableStateOf(Tokens.Palette.accent)
        compose.setContent { Box(Modifier.fillMaxSize().background(Color.White)) { Swatch(colour) } }
        compose.waitForIdle()
        val view = compose.activity.window.decorView
        val accent = Snapshots.picture(view)
        assertEquals("the same view is the same picture", 0.0, Snapshots.difference(accent, Snapshots.picture(view)).first, 0.0)

        // Another colour in the same place.
        colour = Tokens.Palette.secure
        compose.waitForIdle()
        val changed = Snapshots.difference(Snapshots.picture(view), accent).first
        val swatch = 60.0 * 60.0 / (view.width * view.height)
        assertEquals("every pixel of the swatch differs, and no other", swatch, changed, 0.01)
        assertTrue("a recoloured swatch is a failure", changed > Snapshots.SHARE_TOLERANCE)
    }

    @Composable
    private fun Swatch(colour: TokenColor) {
        Box(Modifier.size(60.dp).background(colour.light))
    }
}
