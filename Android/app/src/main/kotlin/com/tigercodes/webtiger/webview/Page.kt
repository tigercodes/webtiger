// The seam between the engine adapter's logic and the WebView itself, so the first is tested on
// the JVM without the second.
package com.tigercodes.webtiger.webview

import android.view.View
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.NavigationCommand

/** One web view, as the engine holds it. [WebViewPage] is the real one. */
internal interface Page {
    /** The view a window shows for the page. Null when there is none: after its renderer died, or in a test. */
    val view: View?

    /** Loads an address. False when no web view could be made to load it in. */
    fun load(url: String): Boolean

    /** Back, forward, reload or stop. Nothing happens when the web view has nowhere to go. */
    fun command(command: NavigationCommand)

    /**
     * Finds text in the page and shows the match (010): the first, or with the same text again
     * the next, or the one before with `backwards`. Empty text clears what was found.
     */
    fun find(text: String, backwards: Boolean)

    /**
     * Sets the page's zoom, in per cent (012): it stays, whatever page the web view goes on to,
     * until it is set again.
     */
    fun zoom(percent: Int)

    /** Destroys the web view and releases what it held. */
    fun close()
}

/** What a page needs of the engine: where its reports go, and who answers its questions. */
internal interface PageHost {
    fun report(event: EngineEvent)

    fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict

    /** Called on the WebView's own threads. */
    fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict

    /**
     * The page of the web view `askingId` asked for a new window (006). The engine makes the
     * page the window's web view is, in the asking page's login set, and reports it; what
     * becomes of it is the core's to say. Null when none could be made.
     */
    fun newWindow(askingId: Long, userGesture: Boolean): Page?

    /** The page of the web view `webViewId` offered something to be downloaded (013). */
    fun download(webViewId: Long, offer: DownloadOffer)
}

/** Makes pages. [WebViewPages] makes real ones. */
internal interface PageFactory {
    /** Whether this device's WebView can keep one login set's data apart from another's. */
    val keepsLoginSetsApart: Boolean

    /** A page whose web view runs in the login set `loginSet`, a UUID. Throws when the device has no usable WebView. */
    fun create(webViewId: Long, loginSet: String, host: PageHost): Page
}
