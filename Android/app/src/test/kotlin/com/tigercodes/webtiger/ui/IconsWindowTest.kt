// Sites' icons (046), the window's part: the pictures of the numbers the core gives tabs'
// icons. The window's model against a scripted core, and the row drawn by Compose on
// Robolectric's Android with its real pictures: what is asked of the core, once, and what a
// tab and a tile show.
package com.tigercodes.webtiger.ui

import android.graphics.Bitmap
import android.graphics.Color
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.Icon
import com.tigercodes.webtiger.core.v1.IconsPage
import com.tigercodes.webtiger.core.v1.Layout
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.io.ByteArrayOutputStream
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import okio.ByteString.Companion.toByteString
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class IconsWindowTest {
    @get:Rule val compose = createComposeRule()

    /** What the scripted core has for each number: a picture, or bytes that are none. */
    private val kept = HashMap<Long, ByteArray>()

    private fun script(request: Request): Response {
        val asked = request.query_icons ?: return windowScript(request)
        return Response(icons_page = IconsPage(icons = asked.icon_ids.mapNotNull { number -> kept[number]?.let { Icon(icon_id = number, png = it.toByteString()) } }))
    }

    private val core = ScriptedCore(::script)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** A PNG of one colour, this many pixels each way. */
    private fun png(colour: Int, size: Int = 32): ByteArray {
        val picture = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888).also { it.eraseColor(colour) }
        return ByteArrayOutputStream().also { picture.compress(Bitmap.CompressFormat.PNG, 100, it) }.toByteArray()
    }

    private fun tab(id: Long, icon: Long = 0, favourite: Boolean = false, kept: Boolean = favourite, asleep: Boolean = false, title: String = "Tab $id"): TabSummary =
        TabSummary(tab_id = id, title = title, host = "example.com", kept = kept, favourite = favourite, asleep = asleep, site_name = if (kept) title else "", icon_id = icon)

    /** The window as the core says it is: tab 2 in front with a page, and these tabs. */
    private fun said(tabs: List<TabSummary>): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "Tab 2", address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = tabs,
        )

    /** The numbers a core was asked the pictures of, question by question. */
    private fun asked(core: ScriptedCore = this.core): List<List<Long>> = core.requests.mapNotNull { it.query_icons?.icon_ids }

    @Test
    fun theWindowAsksForThePicturesItHasNotGotOnceEach() =
        runTest {
            kept[7] = png(Color.RED)
            kept[8] = png(Color.BLUE, size = 64)
            kept[11] = "not a picture at all".toByteArray()
            kept[12] = png(Color.GREEN, size = 300)
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            // Tabs with icons 7 and 8, one of them twice, and a tab with none: one question,
            // for the two numbers, and the window has both pictures.
            core.emit(said(listOf(tab(2, icon = 8), tab(5, icon = 7), tab(6, icon = 7), tab(9))), subscription = 3)
            assertEquals(listOf(listOf(7L, 8L)), asked(core))
            assertEquals(setOf(7L, 8L), model.icons.keys)
            assertEquals(32, model.icons[7]?.width)
            assertEquals(64, model.icons[8]?.width)
            // Told the same again, it asks nothing: it has them.
            core.emit(said(listOf(tab(2, icon = 8), tab(5, icon = 7))), subscription = 3)
            assertEquals(1, asked(core).size)
            // A number it has not is asked for, alone. One the core has no picture of, bytes
            // that are no picture, and a picture larger than the core keeps, are not drawn;
            // and none of them is asked for again.
            core.emit(said(listOf(tab(2, icon = 8), tab(5, icon = 9), tab(6, icon = 11), tab(9, icon = 12))), subscription = 3)
            assertEquals(listOf(listOf(7L, 8L), listOf(9L, 11L, 12L)), asked(core))
            assertEquals(setOf(7L, 8L), model.icons.keys)
            core.emit(said(listOf(tab(2, icon = 9), tab(5, icon = 11))), subscription = 3)
            assertEquals(2, asked(core).size)
        }

    @Test
    fun noMoreIsAskedInOneQuestionThanAnAnswerCarries() =
        runTest {
            for (number in 1L..70L) kept[number] = png(Color.RED, size = 2)
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said((1L..70L).map { tab(100 + it, icon = it) }), subscription = 3)
            assertEquals(listOf(64, 6), asked(core).map { it.size })
            assertEquals(70, model.icons.size)
        }

    @Test
    fun aWindowThatHasKeptManyPicturesKeepsOnlyThoseItsTabsShow() =
        runTest {
            for (number in 1L..610L) kept[number] = png(Color.RED, size = 2)
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            // Six hundred icons, a hundred tabs at a time: the window keeps them all, as it
            // goes, for tabs it may show again.
            for (first in 1L..600L step 100) {
                core.emit(said((first until first + 100).map { tab(1000 + it, icon = it) }), subscription = 3)
            }
            assertEquals(600, model.icons.size)
            // It has more than it keeps: told of other tabs, it keeps the pictures of those
            // it shows now, and lets the rest go.
            core.emit(said((601L..610L).map { tab(1000 + it, icon = it) }), subscription = 3)
            assertEquals((601L..610L).toSet(), model.icons.keys)
            // A tab shows an icon the window let go of: it is asked for again, and is back.
            val questions = asked(core).size
            core.emit(said(listOf(tab(1001, icon = 1), tab(1601, icon = 601))), subscription = 3)
            assertEquals(questions + 1, asked(core).size)
            assertEquals(listOf(1L), asked(core).last())
            assertNotNull(model.icons[1])
            assertNotNull(model.icons[601])
        }

    @Test
    fun whatIsNoPictureOfASizeTheCoreKeepsIsNotDrawn() {
        assertNotNull(iconPicture(png(Color.RED)))
        assertNotNull(iconPicture(png(Color.RED, size = 256)))
        assertNull(iconPicture(png(Color.RED, size = 257)))
        assertNull(iconPicture(ByteArray(0)))
        assertNull(iconPicture("GIF89a and nothing after".toByteArray()))
    }

    /** In a window wide enough for the whole row to be in view at once. */
    @Test
    @Config(qualifiers = "w720dp-h900dp")
    fun aTabHasItsIconBeforeItsNameAndATileIsItsIcon() {
        kept[7] = png(Color.RED)
        kept[8] = png(Color.BLUE)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        // Two Favourites, one whose site has an icon and one whose has none; a Site with an
        // icon; and a tab with none.
        compose.runOnIdle {
            core.emit(
                said(
                    listOf(
                        tab(5, icon = 7, favourite = true, title = "Mail"),
                        tab(6, favourite = true, asleep = true, title = "Board"),
                        tab(7, icon = 8, kept = true, title = "Wiki"),
                        tab(2, title = "News"),
                    ),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        compose.onAllNodesWithTag("tab").assertCountEquals(4)
        // The tile with an icon is its icon, and has no letter; the other has its letter.
        // The Site has its icon, and keeps its mark; the tab has neither.
        compose.onAllNodesWithTag("tab-icon", useUnmergedTree = true).assertCountEquals(2)
        compose.onAllNodesWithTag("favourite-letter", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("site-mark", useUnmergedTree = true).assertCountEquals(1)
        val icons = compose.onAllNodesWithTag("tab-icon", useUnmergedTree = true).fetchSemanticsNodes()
        val tabs = compose.onAllNodesWithTag("tab").fetchSemanticsNodes()
        // Each icon is in its own tab: the first tile's, and the Site's, before its name.
        assertEquals("the first icon is in the first tile", true, tabs[0].boundsInRoot.contains(icons[0].boundsInRoot.center))
        assertEquals("the second icon is in the Site's tab", true, tabs[2].boundsInRoot.contains(icons[1].boundsInRoot.center))
        assertEquals("a tile's icon is larger than a tab's", true, icons[0].size.width > icons[1].size.width)
    }

    @Test
    @Config(qualifiers = "w720dp-h900dp")
    fun downTheSideATileAndARowHaveTheirIcons() {
        kept[7] = png(Color.RED)
        kept[8] = png(Color.BLUE)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                said(
                    listOf(
                        tab(5, icon = 7, favourite = true, title = "Mail"),
                        tab(6, favourite = true, title = "Board"),
                        tab(7, icon = 8, kept = true, title = "Wiki"),
                        tab(2, title = "News"),
                    ),
                ).copy(layout = Layout.LAYOUT_SIDEBAR),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        // The tile with an icon and the Site's row have theirs; the other tile has its letter.
        compose.onAllNodesWithTag("tab-icon", useUnmergedTree = true).assertCountEquals(2)
        compose.onAllNodesWithTag("favourite-letter", useUnmergedTree = true).assertCountEquals(1)
    }
}
