package com.tigercodes.webtiger.webview

import android.webkit.WebView
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature

/**
 * Global Privacy Control (069) in the System WebView: the signal that says "do not sell or
 * share what you learn of me". A page's scripts read it as `navigator.globalPrivacyControl`;
 * a server reads it as the header `Sec-GPC: 1`.
 *
 * The WebView has no setting for either. The property is made by a script of the shell's, run
 * before any of a page's own, in every frame. The header is added to the loads the shell
 * itself begins; what a page fetches of its own accord goes without it.
 */
internal object PrivacySignal {
    /** Makes the property, as one the page cannot write, take away or make again. */
    const val SCRIPT =
        "(function () { try { Object.defineProperty(navigator, 'globalPrivacyControl', " +
            "{ value: true, writable: false, configurable: false, enumerable: true }); } catch (e) {} })();"

    /** What a load the shell begins carries. */
    val HEADERS: Map<String, String> = mapOf("Sec-GPC" to "1")

    /**
     * Has every page and frame of a web view told the signal, where the WebView can run a
     * script before a page's own. One that cannot leaves its pages untold: the header is sent
     * all the same.
     */
    fun tell(view: WebView) {
        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {
            WebViewCompat.addDocumentStartJavaScript(view, SCRIPT, setOf("*"))
        }
    }
}
