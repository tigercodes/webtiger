// The seam between the engine adapter's logic and the WebView itself, so the first is tested on
// the JVM without the second.
package com.tigercodes.webtiger.webview

import android.view.View
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.PermissionKind

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

    /**
     * Offers the file at `url` as a download the person chose in a menu (022): what a link
     * leads to, or a picture. Nothing is fetched until the core has said.
     */
    fun save(url: String)

    /** Destroys the web view and releases what it held. */
    fun close()
}

/**
 * What the person pressed and held in a page (018, 022): where the link leads, and the
 * picture's address. A link, a picture, or a picture that is a link.
 */
data class Pressed(val link: String = "", val picture: String = "")

/** An address of the web: one a tab can start from, and the system's downloader can fetch. */
internal fun isOfTheWeb(url: String): Boolean = url.startsWith("http://", ignoreCase = true) || url.startsWith("https://", ignoreCase = true)

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

    /**
     * A page of the web view `webViewId` asks, as `origin`, to use something of the device's
     * (014). `answer` tells the page yes or no, once the core has said. The adapter's name for
     * the request, to withdraw it by.
     */
    fun permissionAsked(webViewId: Long, origin: String, kinds: List<PermissionKind>, answer: (Boolean) -> Unit): Long

    /** The page no longer waits for an answer to that request. */
    fun permissionWithdrawn(request: Long)

    /**
     * A page of the web view `webViewId` says `message`, or asks, from a frame at `url` (016).
     * `answer` gives the page its answer, once the core has said: whether it was accepted, and
     * for a prompt the text.
     */
    fun dialog(webViewId: Long, url: String, kind: DialogKind, message: String, defaultText: String, answer: (Boolean, String) -> Unit)

    /**
     * A server asks who is there in the web view `webViewId` (017): its `host` and `realm`,
     * where the web view's load was `heading`, and whether the server refused an answer
     * already. `answer` gives the engine the server's answer, once the core has said: what was
     * typed, or null for none.
     */
    fun signIn(webViewId: Long, host: String, realm: String, heading: String, again: Boolean, answer: (Typed?) -> Unit)

    /**
     * The person pressed and held a link or a picture of the page of the web view `webViewId`
     * (018, 022): where the link leads and the picture's address, as the engine gives them.
     * The press is not the page's any more; what comes of it is the menu.
     */
    fun pressed(webViewId: Long, pressed: Pressed)

    /**
     * A page of the web view `webViewId` asks for a file (019). `give` gives the engine what
     * was picked, once: the files' addresses, or null for none.
     */
    fun files(webViewId: Long, asked: FilesAsked, give: (List<String>?) -> Unit)
}

/** Makes pages. [WebViewPages] makes real ones. */
internal interface PageFactory {
    /** Whether this device's WebView can keep one login set's data apart from another's. */
    val keepsLoginSetsApart: Boolean

    /** A page whose web view runs in the login set `loginSet`, a UUID. Throws when the device has no usable WebView. */
    fun create(webViewId: Long, loginSet: String, host: PageHost): Page
}
