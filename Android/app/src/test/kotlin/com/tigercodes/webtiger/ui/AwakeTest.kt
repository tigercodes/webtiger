// What is awake (056) in Android's window, against a scripted core: what the list asks and
// shows, what "Sleep All Others" sends, and the words for why a tab is awake and where it is.
// The shell decides nothing about which tabs sleep: every test here is of what is asked of the
// core, and of what is shown of its answers. (What the core answers is the core's, awake.rs;
// that the real core's answer reaches this shell is ThroughTheCoreTest's.)
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AwakePage
import com.tigercodes.webtiger.core.v1.AwakeTab
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.KeepAwake
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** What a scripted core says is awake; "sleep all others" takes off it what the core would. */
internal class ScriptedAwake(var page: AwakePage = AwakePage()) {
    fun answer(request: Request): Response =
        when {
            request.query_awake != null -> Response(awake_page = page)
            request.sleep_others != null -> {
                // The core puts to sleep what can be: the tabs out of sight with no reason.
                val slept = page.tabs.filter { !it.in_sight && it.keeps_awake.isEmpty() }
                page = page.copy(tabs = page.tabs - slept.toSet(), asleep = page.asleep + slept.size)
                Response(ack = Ack())
            }
            else -> windowScript(request)
        }
}

internal fun awakeTab(
    id: Long,
    title: String,
    host: String = "example.com",
    space: String = "",
    inSight: Boolean = false,
    privateWindow: Boolean = false,
    reasons: List<KeepAwake> = emptyList(),
) = AwakeTab(tab_id = id, window_id = 7, title = title, host = host, space_name = space, in_sight = inSight, private_window = privateWindow, keeps_awake = reasons)

@RunWith(RobolectricTestRunner::class)
class AwakeTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val told = ScriptedAwake()
    private val core = ScriptedCore(told::answer)

    @After
    fun end() {
        scope.cancel()
    }

    private fun model() = AwakeModel(BrowserSession(core, SystemWebViewEngine(FakePages()), scope), windowId = 7)

    /** What was asked of the core about what is awake, in order, in a few words each. */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.query_awake?.let { "what is awake, for ${it.window_id}" } ?: request.sleep_others?.let { "sleep others, from ${it.window_id}" }
        }

    private val three =
        listOf(
            awakeTab(3, "Third", inSight = true),
            awakeTab(1, "First"),
            awakeTab(2, "Second", reasons = listOf(KeepAwake.KEEP_AWAKE_SOUND)),
        )

    @Test
    fun theListIsAskedOfTheWindowAndShownInTheCoresOrderWithItsCounts() {
        told.page = AwakePage(tabs = three, private_tabs = 2, asleep = 9)
        val model = model()
        assertFalse("nothing is said before the core has answered", model.loaded)
        assertFalse(model.othersCouldSleep)

        model.reload()
        assertTrue(model.loaded)
        assertEquals("in the core's order, and no other", listOf("Third", "First", "Second"), model.tabs.map { it.title })
        assertEquals(2, model.privateTabs)
        assertEquals(9, model.asleep)
        assertEquals(listOf("what is awake, for 7"), asked())
    }

    @Test
    fun sleepAllOthersIsAskedOfTheCoreAndTheListIsAskedForAgain() {
        told.page = AwakePage(tabs = three, asleep = 4)
        val model = model()
        model.reload()
        assertTrue("one is out of sight with nothing to keep it awake", model.othersCouldSleep)

        model.sleepOthers()
        // The command, and then the question: in that order, so the answer is of what is left.
        assertEquals(listOf("what is awake, for 7", "sleep others, from 7", "what is awake, for 7"), asked())
        assertEquals("what the core left awake: the shell took none off itself", listOf("Third", "Second"), model.tabs.map { it.title })
        assertEquals(5, model.asleep)
        assertFalse("the one in sight and the one with a reason are left: there are no others", model.othersCouldSleep)
    }

    @Test
    fun theButtonIsWorthPressingOnlyWhenSomeTabCouldSleep() {
        val model = model()
        val cases =
            listOf(
                emptyList<AwakeTab>() to false,
                listOf(awakeTab(1, "In sight", inSight = true)) to false,
                listOf(awakeTab(1, "With a reason", reasons = listOf(KeepAwake.KEEP_AWAKE_TYPED))) to false,
                // A private window's tab does not sleep (037), whatever else is so of it.
                listOf(awakeTab(1, "Private", privateWindow = true)) to false,
                listOf(awakeTab(1, "In sight", inSight = true), awakeTab(2, "Behind")) to true,
            )
        for ((tabs, could) in cases) {
            told.page = AwakePage(tabs = tabs)
            model.reload()
            assertEquals(tabs.map { it.title }.toString(), could, model.othersCouldSleep)
        }
    }

    /** A core whose answers about what is awake wait until the test lets each go, in whatever order it likes. */
    private class HeldCore : CoreClient {
        val held = ArrayList<CompletableDeferred<Response>>()
        override val events: Flow<Event> = emptyFlow()

        override suspend fun request(request: Request): Response {
            if (request.query_awake == null) return windowScript(request)
            return CompletableDeferred<Response>().also(held::add).await()
        }

        override fun report(event: EngineEvent) = Unit

        override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean) = NavigationVerdict.ALLOW

        override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType) = RequestVerdict.ALLOW

        override fun stop() = Unit
    }

    @Test
    fun anAnswerThatArrivesAfterALaterQuestionsIsNotShown() {
        val slow = HeldCore()
        val model = AwakeModel(BrowserSession(slow, SystemWebViewEngine(FakePages()), scope), windowId = 7)
        // Asked twice, and the second question is answered first.
        model.reload()
        model.reload()
        assertEquals(2, slow.held.size)
        slow.held[1].complete(Response(awake_page = AwakePage(tabs = listOf(awakeTab(2, "Now")), asleep = 2)))
        assertTrue(model.loaded)
        assertEquals(listOf("Now"), model.tabs.map { it.title })

        // The first question's answer arrives late: it is of what was so before, and is not shown.
        slow.held[0].complete(Response(awake_page = AwakePage(tabs = listOf(awakeTab(1, "Then")), asleep = 1)))
        assertEquals("an answer to an older question is out of date", listOf("Now"), model.tabs.map { it.title })
        assertEquals(2, model.asleep)
    }

    @Test
    fun aCoreThatWillNotAnswerLeavesTheListAsItWas() {
        val refusing = ScriptedCore { request -> if (request.query_awake != null) throw com.tigercodes.webtiger.core.CoreError.Refused("window.unknown", "window_id") else windowScript(request) }
        val model = AwakeModel(BrowserSession(refusing, SystemWebViewEngine(FakePages()), scope), windowId = 7)
        model.reload()
        assertFalse(model.loaded)
        assertEquals(emptyList<AwakeTab>(), model.tabs)
    }

    @Test
    fun whyATabIsAwakeIsSaidInTheCoresOrderAndOnScreenFirst() {
        val words = mapOf(
            R.string.awake_why_sound to "Playing sound", R.string.awake_why_capture to "Using the camera or microphone",
            R.string.awake_why_typed to "Has text you typed", R.string.awake_why_download to "Downloading",
            R.string.awake_why_question to "Asking you something", R.string.awake_why_loading to "Loading",
        )
        val why = { tab: AwakeTab -> awakeWhy(tab, awakeReasons(tab).map { words.getValue(it) }, onScreen = "On screen", waiting = "Will sleep by itself") }
        assertEquals("On screen", why(awakeTab(1, "A", inSight = true)))
        assertEquals("Will sleep by itself", why(awakeTab(1, "A")))
        assertEquals("Playing sound", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_SOUND))))
        assertEquals("Using the camera or microphone", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_CAPTURE))))
        assertEquals("Has text you typed", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_TYPED))))
        assertEquals("Downloading", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_DOWNLOAD))))
        assertEquals("Asking you something", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_QUESTION))))
        assertEquals("Loading", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_LOADING))))
        assertEquals(
            "On screen, Playing sound, Has text you typed, Loading",
            why(awakeTab(1, "A", inSight = true, reasons = listOf(KeepAwake.KEEP_AWAKE_SOUND, KeepAwake.KEEP_AWAKE_TYPED, KeepAwake.KEEP_AWAKE_LOADING))),
        )
        // A reason with no words is not said, and is not taken for none: nothing is said of a
        // tab whose only reason is one, rather than that it will sleep.
        assertEquals("", why(awakeTab(1, "A", reasons = listOf(KeepAwake.KEEP_AWAKE_UNSPECIFIED))))
        assertEquals("On screen", why(awakeTab(1, "A", inSight = true, reasons = listOf(KeepAwake.KEEP_AWAKE_UNSPECIFIED))))
    }

    @Test
    fun everyReasonTheContractHasHasWords() {
        for (reason in KeepAwake.entries) {
            if (reason == KeepAwake.KEEP_AWAKE_UNSPECIFIED) assertNull(awakeWord(reason)) else assertNotNull("$reason", awakeWord(reason))
        }
        assertEquals("each its own", KeepAwake.entries.size - 1, KeepAwake.entries.mapNotNull(::awakeWord).toSet().size)
    }

    @Test
    fun aTabIsNamedByItsPageOrItsSiteAndPlacedByItsSpace() {
        assertEquals("Example Domain", awakeName(awakeTab(1, "Example Domain"), untitled = "New Tab"))
        assertEquals("example.org", awakeName(awakeTab(1, "", host = "example.org"), untitled = "New Tab"))
        assertEquals("New Tab", awakeName(awakeTab(1, "", host = ""), untitled = "New Tab"))
        // Where it is: the Space by its name, the app's own Space by the shell's word, a
        // private window as that; and the site, where the name is not the site already.
        val place = { tab: AwakeTab -> awakePlace(tab, home = "Home", privateWindow = "Private Window") }
        assertEquals("Work · example.com", place(awakeTab(1, "A", space = "Work")))
        assertEquals("Home · example.com", place(awakeTab(1, "A")))
        assertEquals("Private Window · example.com", place(awakeTab(1, "A", privateWindow = true)))
        assertEquals("Work", place(awakeTab(1, "", space = "Work")))
        assertEquals("Work", place(awakeTab(1, "A", host = "", space = "Work")))
    }

    // The list, drawn

    private var done = 0

    private fun show(page: AwakePage) {
        told.page = page
        val model = model()
        compose.setContent { WebTigerTheme { AwakeList(model, onDone = { done += 1 }) } }
        compose.waitForIdle()
    }

    @Test
    fun eachTabIsShownByItsNameWhereItIsAndWhyItIsAwake() {
        show(AwakePage(tabs = three + awakeTab(4, "", host = "files.example.net", space = "Work", reasons = listOf(KeepAwake.KEEP_AWAKE_DOWNLOAD)), private_tabs = 2, asleep = 9))
        compose.onAllNodesWithTag("awake-tab").assertCountEquals(4)
        val first = compose.onAllNodesWithTag("awake-tab")[0]
        first.assertTextContains("Third")
        first.assertTextContains("Home · example.com")
        first.assertTextContains("On screen")
        compose.onAllNodesWithTag("awake-tab")[1].assertTextContains("Will sleep by itself")
        compose.onAllNodesWithTag("awake-tab")[2].assertTextContains("Playing sound")
        // A page with no name is named by its site, and its place is its Space alone.
        val last = compose.onAllNodesWithTag("awake-tab")[3]
        last.assertTextContains("files.example.net")
        last.assertTextContains("Work")
        last.assertTextContains("Downloading")
        // The counts are under the tabs, at the list's end.
        compose.onNodeWithTag("awake-list").performScrollToNode(hasTestTag("awake-asleep"))
        compose.onNodeWithTag("awake-asleep").assertTextEquals("Asleep: 9")
        compose.onNodeWithTag("awake-private").assertTextEquals("Awake in private windows: 2")
        compose.onNodeWithTag("awake-sleep-others").assertIsEnabled()
    }

    @Test
    fun sleepAllOthersLeavesTheOnesTheCoreLeftAndDoneIsDone() {
        show(AwakePage(tabs = three, asleep = 4))
        compose.onNodeWithTag("awake-sleep-others").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("awake-tab").assertCountEquals(2)
        compose.onNodeWithTag("awake-asleep").assertTextEquals("Asleep: 5")
        compose.onNodeWithTag("awake-sleep-others").assertIsNotEnabled()
        compose.onNodeWithTag("awake-private").assertDoesNotExist()
        compose.onNodeWithTag("awake-done").performClick()
        assertEquals(1, done)
    }

    @Test
    fun withNoTabAwakeTheListSaysSoAndHowManyAreAsleep() {
        show(AwakePage(asleep = 40))
        compose.onNodeWithTag("awake-nothing").assertTextEquals("No tab is awake.")
        compose.onNodeWithTag("awake-asleep").assertTextEquals("Asleep: 40")
        compose.onAllNodesWithTag("awake-tab").assertCountEquals(0)
        compose.onNodeWithTag("awake-sleep-others").assertIsNotEnabled()
    }
}
