// A page's own list of choices (a select), in the app itself on a device, in fake mode. The
// WebView shows such a list in a window of its own over the page, and for that it needs the
// activity the page is shown in: a web view that has only the application has nowhere to put
// it (032). The list is the WebView's own, in the app's process: nothing of the system's.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.view.inspector.WindowInspector
import android.webkit.WebView
import android.widget.AdapterView
import android.widget.TextView
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppChoicesTest {
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

    private fun onMain(block: () -> Unit) = InstrumentationRegistry.getInstrumentation().runOnMainSync(block)

    /** The title of the page the window shows. */
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

    /** The words of every piece of text in the app's windows that are not the window itself: the lists and dialogs over it. */
    private fun wordsOverTheWindow(): List<String> {
        val words = ArrayList<String>()
        onMain {
            val own = scenarioRoot()
            for (root in WindowInspector.getGlobalWindowViews()) {
                if (root !== own) collect(root, words)
            }
        }
        return words
    }

    private var windowRoot: View? = null

    private fun scenarioRoot(): View? = windowRoot

    private fun collect(view: View, words: MutableList<String>) {
        if (view is TextView && view.text.isNotBlank()) words += view.text.toString()
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) collect(view.getChildAt(index), words)
        }
    }

    /** Presses the piece of text that says `words`, in a window over the app's own. False when there is none. */
    private fun press(words: String): Boolean {
        var pressed = false
        onMain {
            for (root in WindowInspector.getGlobalWindowViews()) {
                if (root === windowRoot) continue
                find(root, words)?.let { found ->
                    // The words are in a row of a list: it is the list that takes a press on
                    // a row. Where they are in no list, whatever they are in takes it.
                    var row: View = found
                    var holder = row.parent
                    while (holder is View && holder !is AdapterView<*>) {
                        row = holder
                        holder = holder.parent
                    }
                    if (holder is AdapterView<*>) {
                        val place = holder.getPositionForView(row)
                        pressed = holder.performItemClick(row, place, holder.getItemIdAtPosition(place))
                    } else {
                        var target: View? = found
                        while (target != null && !target.performClick()) target = target.parent as? View
                        pressed = target != null
                    }
                }
            }
        }
        return pressed
    }

    private fun find(view: View, words: String): View? {
        if (view is TextView && view.text.toString() == words) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) find(view.getChildAt(index), words)?.let { return it }
        }
        return null
    }

    @Test
    fun aPagesListOfChoicesOpensOverThePageAndTheChoiceIsThePages() {
        awaitPage("Home")
        // The page with the list of choices, typed in the address field.
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url("/choose"))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage("Choose")
        scenario.onActivity { activity -> windowRoot = activity.window.decorView }
        val page = checkNotNull(run { var found: WebView? = null; scenario.onActivity { found = webViewIn(it.window.decorView) }; found })

        // A finger on the page's list: its choices come up over the page, in the WebView's own
        // window, which it can open only because it has the activity the page is shown in.
        PageTouch.tap(page, "pick")
        try {
            compose.waitUntil(15_000) { wordsOverTheWindow().contains("A lion") }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // The test page's own choices, nobody's.
            throw AssertionError("the page's list of choices never came up; over the window there is ${wordsOverTheWindow()}")
        }
        assert(wordsOverTheWindow().containsAll(listOf("Nothing", "A tiger", "A lion"))) { "the choices shown are ${wordsOverTheWindow()}" }

        // One is chosen: the list goes, and the page has the choice.
        assert(press("A lion")) { "the choice could not be pressed" }
        awaitPage("chose lion")
        compose.waitUntil(15_000) { wordsOverTheWindow().isEmpty() }
    }
}
