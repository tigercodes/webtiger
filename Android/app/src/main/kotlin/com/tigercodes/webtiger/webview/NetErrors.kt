// The System WebView's load errors as the contract's codes (engine.proto, NavigationFailed).
package com.tigercodes.webtiger.webview

import android.webkit.WebViewClient

/**
 * A main-frame load error as one of the contract's codes. The engine's own description of the
 * error never leaves the shell.
 *
 * The WebView has no error of its own for "no network": it reports a failed host lookup. So a
 * lookup that fails while the device has no network at all ([online] false) is `net.offline`.
 * A cancelled load is never reported as an error by the WebView; [PageTracker] reports those.
 */
internal fun netErrorCode(errorCode: Int, online: Boolean): String =
    when (errorCode) {
        WebViewClient.ERROR_HOST_LOOKUP -> if (online) "net.dns" else "net.offline"
        WebViewClient.ERROR_CONNECT, WebViewClient.ERROR_IO -> "net.connect"
        WebViewClient.ERROR_TIMEOUT -> "net.timeout"
        WebViewClient.ERROR_FAILED_SSL_HANDSHAKE -> "net.tls"
        else -> "net.failed"
    }
