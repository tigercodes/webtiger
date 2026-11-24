// Pages with no WebView behind them, for the tests of everything above the WebView.
package com.tigercodes.webtiger.webview

import android.view.View
import com.tigercodes.webtiger.core.v1.NavigationCommand

/** A page that records what it was asked to do. */
internal class FakePage(
    val id: Long,
    val loginSet: String,
    val host: PageHost,
    /** Whether it was made as a private window's page, whose keyboard is asked not to learn (037 §5.5). */
    val forgetful: Boolean = false,
) : Page {
    val loaded = ArrayList<String>()
    val commands = ArrayList<NavigationCommand>()
    var closed = false
    var loads = true

    /** What a real page keeps: the tracker its web view's callbacks would go to. */
    val tracker = PageTracker(id, host::report)

    override val view: View? = null

    override fun load(url: String): Boolean {
        if (!loads) return false
        loaded.add(url)
        tracker.willLoad(url)
        return true
    }

    override fun command(command: NavigationCommand) {
        commands.add(command)
    }

    /** What the page gives as its state: none, until a test says. */
    var hasState: ByteArray? = null

    /** How many times the page was asked for its state. */
    var statesAsked = 0

    override fun state(): ByteArray? {
        statesAsked += 1
        return hasState
    }

    /** The states the page was handed, each with the address it came with. */
    val restored = ArrayList<Pair<String, String>>()

    /** Whether the page takes a state it is handed: one that does not loads the address. */
    var takesStates = true

    override fun restore(state: ByteArray, url: String): Boolean {
        if (!loads) return false
        restored.add(String(state) to url)
        if (!takesStates) return load(url)
        tracker.willLoad(url)
        return true
    }

    /** What the page was asked to find, in order: the text (empty to stop), and whether backwards. */
    val finds = ArrayList<Pair<String, Boolean>>()

    override fun find(text: String, backwards: Boolean) {
        finds.add(text to backwards)
    }

    /** Every zoom the page was given, in order. */
    val zooms = ArrayList<Int>()

    override fun zoom(percent: Int) {
        zooms.add(percent)
    }

    /** The addresses the page was asked to save, in order. */
    val saved = ArrayList<String>()

    /** As a real page does: the file is offered as a download the person chose, with no name suggested. */
    override fun save(url: String) {
        saved.add(url)
        host.download(id, DownloadOffer(url = url, userAgent = "Agent/1", suggestedName = "", mimeType = "", length = 0, chosen = true))
    }

    override fun close() {
        closed = true
    }
}

/** Makes [FakePage]s, as a device whose WebView has profiles (or, told otherwise, has none). */
internal class FakePages(override var keepsLoginSetsApart: Boolean = true) : PageFactory {
    val made = ArrayList<FakePage>()
    var failure: RuntimeException? = null

    override fun create(webViewId: Long, loginSet: String, forgetful: Boolean, host: PageHost): Page {
        failure?.let { throw it }
        did.add("make $loginSet")
        return FakePage(webViewId, loginSet, host, forgetful).also { made.add(it) }
    }

    /** What the adapter did about private login sets, in order: "sweep", "throw away <name>", "clear <name>"; and "make <name>" for each page made. */
    val did = ArrayList<String>()

    /** Whether a private login set can be written down as one to throw away. */
    var remembers = true

    override fun willThrowAway(loginSet: String): Boolean {
        if (remembers) did.add("throw away $loginSet")
        return remembers
    }

    override fun clear(loginSet: String) {
        did.add("clear $loginSet")
    }

    override fun sweep() {
        did.add("sweep")
    }
}
