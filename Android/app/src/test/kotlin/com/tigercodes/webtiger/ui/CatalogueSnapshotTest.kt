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
private const val TALL_LIGHT = "w411dp-h900dp-notnight-xhdpi"
private const val TALL_DARK = "w411dp-h900dp-night-xhdpi"

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

    private fun windows(theme: String) {
        var state by mutableStateOf(CatalogueState.EMPTY)
        show { key(state) { CatalogueWindow(state, Modifier.fillMaxSize()) } }
        for (next in CatalogueState.entries) {
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

    @Test @Config(qualifiers = LIGHT)
    fun everyStateOfTheHistoryListInTheLightTheme() = historyLists("light")

    @Test @Config(qualifiers = DARK)
    fun everyStateOfTheHistoryListInTheDarkTheme() = historyLists("dark")

    @Test @Config(qualifiers = LIST_LIGHT)
    fun everyStateOfTheDownloadsListInTheLightTheme() = downloadsLists("light")

    @Test @Config(qualifiers = LIST_DARK)
    fun everyStateOfTheDownloadsListInTheDarkTheme() = downloadsLists("dark")

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
