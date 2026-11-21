// Where a file the person chose is, and what its site says of it, before any of it is fetched
// (022).
//
// A download a page starts comes with all of that: the WebView has followed the address to
// where the file is, and says its kind, its name and its size. A file chosen in a menu is only
// an address. Two things come of that. The core, which makes the file's name, would not know
// its kind: a picture whose address has no ending would be saved without one. And the system's
// downloader, which is handed the login set's cookies for the address, sends them on to every
// site the address sends it to: seen on the emulator, where a file of one site that sent the
// downloader on to another was fetched there with the first site's cookies.
//
// So the adapter asks first, without fetching: each site on the way with that site's own
// cookies and no other's, following by hand whoever sends it on. What it then offers is where
// the file is, with what the site there says of it. (The system's downloader is gone since
// (027): the adapter fetches the file itself, the same way, AppFetcher.kt. The asking stays,
// for what the core needs to know before it names the file.)
package com.tigercodes.webtiger.webview

import android.os.Handler
import android.os.Looper
import android.webkit.CookieManager
import android.webkit.URLUtil
import androidx.webkit.ProfileStore
import androidx.webkit.WebViewFeature
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URI
import java.net.URISyntaxException
import java.net.URL
import java.util.concurrent.Executor
import java.util.concurrent.Executors

/** Where a file is, once everybody who sends on has been followed, and what its site says of it. */
internal data class Told(
    val url: String,
    val mimeType: String = "",
    /** What the site suggests the file is called. Empty when it suggests nothing. */
    val suggestedName: String = "",
    /** How big the site says the file is. Zero when it did not say. */
    val length: Long = 0,
)

/** One site's answer to being asked after an address, with nothing of the file fetched. */
internal data class Answer(
    val status: Int,
    /** Where it sends whoever asked, when it sends them on. */
    val location: String? = null,
    val mimeType: String = "",
    /** What it says the file is to be called, as it said it. */
    val disposition: String? = null,
    val length: Long = -1,
)

/** Asks after a file before it is fetched. [SystemAsker] is the real one. */
internal fun interface Asker {
    /** Asks after `url` as `userAgent`, in `loginSet`. `answer` is given where the file is and what is said of it: once, on the main thread. */
    fun ask(url: String, userAgent: String, loginSet: String, answer: (Told) -> Unit)
}

/** An asker for an engine that is given none: nothing is asked, and the file is where its address says. */
internal object NoAsker : Asker {
    override fun ask(url: String, userAgent: String, loginSet: String, answer: (Told) -> Unit) = answer(Told(url))
}

/** How many times an address may send on before the asking stops where it is. */
internal const val MOST_HOPS = 10

/**
 * Follows `url` to where its file is. Each address on the way is asked with its own cookies
 * (`cookiesFor`) and nobody else's, by `ask`, which fetches nothing and follows nobody. It
 * stops where an answer sends nowhere: with what was said there, when the answer was a file's;
 * where nobody answered; where the next address would not be the web's; and after
 * [MOST_HOPS]. `name` makes of what a site says a file is to be called the name it suggests.
 */
internal fun whereTheFileIs(
    url: String,
    cookiesFor: (String) -> String?,
    ask: (address: String, cookies: String?) -> Answer?,
    name: (address: String, disposition: String, mimeType: String) -> String,
): Told {
    var at = url
    repeat(MOST_HOPS) {
        val answer = ask(at, cookiesFor(at)) ?: return Told(at)
        val next = answer.location?.takeIf { answer.status in 300..399 }?.let { resolved(at, it) }
        if (next == null) {
            if (answer.status !in 200..299) return Told(at)
            val suggested = answer.disposition?.takeIf { it.isNotBlank() }?.let { name(at, it, answer.mimeType) }.orEmpty()
            return Told(at, answer.mimeType, suggested, answer.length.coerceAtLeast(0))
        }
        if (!isOfTheWeb(next)) return Told(at)
        at = next
    }
    return Told(at)
}

/** Where an answer sends on, as a whole address: it may have said only a path. Null for what is no address, and for one that says nothing. */
internal fun resolved(from: String, location: String): String? =
    try {
        location.trim().takeIf { it.isNotEmpty() }?.let { URI(from).resolve(it).toString() }
    } catch (_: URISyntaxException) {
        null
    } catch (_: IllegalArgumentException) {
        null
    }

/** The real asker: the device's own connections, asked for an address's head and no more. */
internal class SystemAsker(private val worker: Executor = Executors.newSingleThreadExecutor()) : Asker {
    private val main = Handler(Looper.getMainLooper())

    override fun ask(url: String, userAgent: String, loginSet: String, answer: (Told) -> Unit) {
        // The login set's cookies, found here on the main thread and read as each site is asked.
        val jar = cookies(loginSet)
        worker.execute {
            val told =
                whereTheFileIs(
                    url,
                    cookiesFor = { address -> jar?.getCookie(address) },
                    ask = { address, cookies -> head(address, userAgent, cookies) },
                    name = { address, disposition, mimeType -> URLUtil.guessFileName(address, disposition, mimeType.ifEmpty { null }) },
                )
            main.post { answer(told) }
        }
    }

    private fun cookies(loginSet: String): CookieManager? =
        try {
            if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                ProfileStore.getInstance().getProfile(loginSet)?.cookieManager
            } else {
                CookieManager.getInstance()
            }
        } catch (_: RuntimeException) {
            null
        }

    /** A site's answer about an address. A site that will not be asked for a head alone is asked for the file, and nothing of the file is read. */
    private fun head(address: String, userAgent: String, cookies: String?): Answer? =
        asked(address, userAgent, cookies, "HEAD")?.let { first ->
            if (first.status == HttpURLConnection.HTTP_BAD_METHOD || first.status == HttpURLConnection.HTTP_NOT_IMPLEMENTED) asked(address, userAgent, cookies, "GET") else first
        }

    private fun asked(address: String, userAgent: String, cookies: String?, method: String): Answer? {
        val connection =
            try {
                URL(address).openConnection() as? HttpURLConnection
            } catch (_: IOException) {
                null
            } ?: return null
        return try {
            connection.requestMethod = method
            // Followed by hand: what is sent to one site is not sent to the next.
            connection.instanceFollowRedirects = false
            connection.connectTimeout = PATIENCE_MS
            connection.readTimeout = PATIENCE_MS
            if (userAgent.isNotEmpty()) connection.setRequestProperty("User-Agent", userAgent)
            if (!cookies.isNullOrEmpty()) connection.setRequestProperty("Cookie", cookies)
            Answer(
                status = connection.responseCode,
                location = connection.getHeaderField("Location"),
                mimeType = connection.contentType.orEmpty().substringBefore(';').trim(),
                disposition = connection.getHeaderField("Content-Disposition"),
                length = connection.contentLengthLong,
            )
        } catch (_: IOException) {
            null
        } catch (_: RuntimeException) {
            null
        } finally {
            connection.disconnect()
        }
    }

    private companion object {
        /** How long a site is waited for. */
        const val PATIENCE_MS = 5_000
    }
}
