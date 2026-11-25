// Marks for what a tab is doing (058): which of the core's reasons has a mark in the row, which
// mark a tab with several has, and what is said of it beside its name. The model of it, and the
// strip drawn by Compose on Robolectric's Android against a scripted core. The shell decides
// nothing about a tab: the core says its reasons (052).
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.KeepAwake
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TabDoingTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun doing(vararg reasons: KeepAwake) = TabDoing.of(TabSummary(tab_id = 1, title = "A page", keeps_awake = reasons.toList()))

    @Test
    fun soundAndTheCameraOrMicrophoneHaveAMarkAndTheOtherReasonsHaveNone() {
        assertEquals(TabDoing.SOUND, doing(KeepAwake.KEEP_AWAKE_SOUND))
        assertEquals(TabDoing.CAPTURE, doing(KeepAwake.KEEP_AWAKE_CAPTURE))
        assertNull(doing())
        // Every reason the contract has is one of the two, or has no mark: a new one gets a
        // mark only when somebody gives it one.
        for (reason in KeepAwake.entries) {
            val marked = reason == KeepAwake.KEEP_AWAKE_SOUND || reason == KeepAwake.KEEP_AWAKE_CAPTURE
            assertEquals("$reason", marked, doing(reason) != null)
        }
    }

    @Test
    fun aTabHasOneMarkAndTheCameraOrMicrophoneComesBeforeSound() {
        // In the core's order (sound, then the camera), and with others among them.
        assertEquals(TabDoing.CAPTURE, doing(KeepAwake.KEEP_AWAKE_SOUND, KeepAwake.KEEP_AWAKE_CAPTURE))
        assertEquals(TabDoing.CAPTURE, doing(KeepAwake.KEEP_AWAKE_CAPTURE, KeepAwake.KEEP_AWAKE_SOUND))
        assertEquals(TabDoing.SOUND, doing(KeepAwake.KEEP_AWAKE_SOUND, KeepAwake.KEEP_AWAKE_TYPED, KeepAwake.KEEP_AWAKE_LOADING))
        assertEquals(TabDoing.CAPTURE, doing(KeepAwake.KEEP_AWAKE_TYPED, KeepAwake.KEEP_AWAKE_CAPTURE, KeepAwake.KEEP_AWAKE_QUESTION))
    }

    @Test
    fun eachMarkHasItsOwnPictureAndItsOwnWords() {
        assertNotEquals(TabDoing.CAPTURE.picture, TabDoing.SOUND.picture)
        assertEquals(R.string.awake_why_sound, TabDoing.SOUND.words)
        assertEquals(R.string.awake_why_capture, TabDoing.CAPTURE.words)
        assertEquals(R.drawable.ic_sound, TabDoing.SOUND.picture)
        assertEquals(R.drawable.ic_capture, TabDoing.CAPTURE.picture)
    }

    /** What each tab of the strip says of itself beside its name. */
    private fun said(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { it.config.getOrNull(SemanticsProperties.StateDescription).orEmpty() }

    /**
     * The strip, as the core says it: a tab that plays a sound in front; a Site, away, whose
     * page holds the camera; a tab in a call, with both; a tab that was typed into and is on
     * its way; and a tab that does nothing.
     */
    @Test
    fun aTabSaysWhatItIsDoingBesideItsNameAndHasItsMarkAndTheOthersHaveNone() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                WindowSnapshot(
                    window_id = 1,
                    tab = TabView(tab_id = 2, web_view_id = 4, title = "Music", address = AddressView(url = "https://music.example.com/", display = "music.example.com")),
                    tabs =
                        listOf(
                            TabSummary(tab_id = 2, title = "Music", host = "music.example.com", keeps_awake = listOf(KeepAwake.KEEP_AWAKE_SOUND)),
                            TabSummary(tab_id = 5, title = "Meeting", host = "meet.example.com", kept = true, away = true, site_name = "Meet", keeps_awake = listOf(KeepAwake.KEEP_AWAKE_CAPTURE)),
                            TabSummary(tab_id = 6, title = "A call", host = "call.example.com", keeps_awake = listOf(KeepAwake.KEEP_AWAKE_SOUND, KeepAwake.KEEP_AWAKE_CAPTURE)),
                            TabSummary(tab_id = 7, title = "A letter", host = "mail.example.com", keeps_awake = listOf(KeepAwake.KEEP_AWAKE_TYPED, KeepAwake.KEEP_AWAKE_LOADING)),
                            TabSummary(tab_id = 8, title = "News", host = "news.example.com"),
                        ),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        assertEquals(
            listOf("Playing sound", "Site, Away, Using the camera or microphone", "Using the camera or microphone", "", ""),
            said(),
        )
        // One speaker, on the first; two recording marks, on the second and the third; a tab
        // with other reasons, and one with none, has no mark.
        compose.onAllNodesWithTag("doing-sound", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("doing-capture", useUnmergedTree = true).assertCountEquals(2)

        // The sound stops, and the camera is given back: nothing is said, and no mark is left.
        compose.runOnIdle {
            core.emit(
                WindowSnapshot(
                    window_id = 1,
                    tab = TabView(tab_id = 2, web_view_id = 4, title = "Music", address = AddressView(url = "https://music.example.com/", display = "music.example.com")),
                    tabs =
                        listOf(
                            TabSummary(tab_id = 2, title = "Music", host = "music.example.com"),
                            TabSummary(tab_id = 5, title = "Meeting", host = "meet.example.com", kept = true, away = true, site_name = "Meet"),
                        ),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        assertEquals(listOf("", "Site, Away"), said())
        compose.onAllNodesWithTag("doing-sound", useUnmergedTree = true).assertCountEquals(0)
        compose.onAllNodesWithTag("doing-capture", useUnmergedTree = true).assertCountEquals(0)
    }
}
