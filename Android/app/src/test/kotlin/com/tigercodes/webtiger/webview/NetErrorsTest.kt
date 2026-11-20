package com.tigercodes.webtiger.webview

import android.net.http.SslError
import android.webkit.WebViewClient
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class NetErrorsTest {
    @Test
    fun theWebViewsErrorsAreTheContractsCodes() {
        val expected =
            mapOf(
                WebViewClient.ERROR_HOST_LOOKUP to "net.dns",
                WebViewClient.ERROR_CONNECT to "net.connect",
                WebViewClient.ERROR_IO to "net.connect",
                WebViewClient.ERROR_TIMEOUT to "net.timeout",
                WebViewClient.ERROR_FAILED_SSL_HANDSHAKE to "net.tls",
                WebViewClient.ERROR_UNKNOWN to "net.failed",
                WebViewClient.ERROR_BAD_URL to "net.failed",
                WebViewClient.ERROR_UNSUPPORTED_SCHEME to "net.failed",
                WebViewClient.ERROR_REDIRECT_LOOP to "net.failed",
                WebViewClient.ERROR_TOO_MANY_REQUESTS to "net.failed",
                WebViewClient.ERROR_AUTHENTICATION to "net.failed",
                WebViewClient.ERROR_PROXY_AUTHENTICATION to "net.failed",
                WebViewClient.ERROR_UNSUPPORTED_AUTH_SCHEME to "net.failed",
                WebViewClient.ERROR_FILE to "net.failed",
                WebViewClient.ERROR_FILE_NOT_FOUND to "net.failed",
                WebViewClient.ERROR_UNSAFE_RESOURCE to "net.failed",
            )
        for ((error, code) in expected) assertEquals("error $error", code, netErrorCode(error, online = true))
    }

    @Test
    fun aCertificateTheWebViewWouldNotTrustSaysWhichKindOfTroubleItWas() {
        val expected =
            mapOf(
                SslError.SSL_UNTRUSTED to "net.tls.untrusted",
                SslError.SSL_EXPIRED to "net.tls.date",
                SslError.SSL_NOTYETVALID to "net.tls.date",
                SslError.SSL_DATE_INVALID to "net.tls.date",
                SslError.SSL_IDMISMATCH to "net.tls.name",
                // Whatever else it was, and a kind this build has not heard of.
                SslError.SSL_INVALID to "net.tls",
                -1 to "net.tls",
                99 to "net.tls",
            )
        for ((error, code) in expected) assertEquals("error $error", code, certificateErrorCode(error))
    }

    @Test
    fun aLookupThatFailsWithNoNetworkIsOffline() {
        assertEquals("net.offline", netErrorCode(WebViewClient.ERROR_HOST_LOOKUP, online = false))
        // Nothing else changes its meaning with the network: a timeout is a timeout.
        assertEquals("net.timeout", netErrorCode(WebViewClient.ERROR_TIMEOUT, online = false))
        assertEquals("net.connect", netErrorCode(WebViewClient.ERROR_CONNECT, online = false))
        assertEquals("net.tls", netErrorCode(WebViewClient.ERROR_FAILED_SSL_HANDSHAKE, online = false))
    }

    @Test
    fun anErrorThisBuildDoesNotKnowIsStillACode() {
        val code = Regex("^[a-z0-9_]+\\.[a-z0-9_]+$")
        for (error in -40..5) {
            for (online in listOf(true, false)) assertTrue(code.matches(netErrorCode(error, online)))
        }
        assertEquals("net.failed", netErrorCode(-999, online = true))
    }
}
