// The certificates a person went on past (026): which an engine lets through, and for whom. Kept
// in memory, for as long as the process lives, and nowhere else.
package com.tigercodes.webtiger.webview

import java.net.URI
import java.security.MessageDigest

/**
 * A certificate the WebView would not trust: whose it was (the scheme, host and port it was
 * presented at) and which (a hash of the certificate itself).
 */
internal data class Refused(val origin: String, val fingerprint: String)

/**
 * What is let through (026 §3): one certificate, at one host and port, in one login set. The
 * same host with another certificate, another host with the same one, and another login set are
 * each refused as before.
 */
internal class Certificates {
    private data class Exception(val loginSet: String, val certificate: Refused)

    private val trusted = HashSet<Exception>()

    /** The certificate each web view last refused for its own page: what it would be told to let through. */
    private val held = HashMap<Long, Refused>()

    /** Whether a certificate is let through, in a login set. */
    fun trusts(loginSet: String, certificate: Refused): Boolean = Exception(loginSet, certificate) in trusted

    /** A web view's own page was refused this certificate: it is held, to be let through if the core says. */
    fun refused(webViewId: Long, certificate: Refused) {
        held[webViewId] = certificate
    }

    /** Lets through, in its login set, what a web view last refused. False when it holds nothing. */
    fun trust(webViewId: Long, loginSet: String): Boolean {
        val certificate = held[webViewId] ?: return false
        trusted.add(Exception(loginSet, certificate))
        return true
    }

    /** The web view is gone, and with it what it held. What was let through stays, for its login set. */
    fun dropped(webViewId: Long) {
        held.remove(webViewId)
    }
}

/**
 * Whose certificate a connection to `url` presents: its scheme, host and port, with the port
 * written also where the scheme implies it, so that two ways of writing an address are one
 * site. Null for what is no address over `https`.
 */
internal fun certificateOrigin(url: String): String? {
    val address =
        try {
            URI(url)
        } catch (_: java.net.URISyntaxException) {
            return null
        }
    if (!address.scheme.equals("https", ignoreCase = true)) return null
    val host = address.host?.lowercase()?.takeIf { it.isNotEmpty() } ?: return null
    val port = if (address.port == -1) 443 else address.port
    return "https://$host:$port"
}

/** A hash of a certificate itself: SHA-256 of its encoding, in hex. */
internal fun certificateFingerprint(encoded: ByteArray): String =
    MessageDigest.getInstance("SHA-256").digest(encoded).joinToString("") { "%02x".format(it) }
