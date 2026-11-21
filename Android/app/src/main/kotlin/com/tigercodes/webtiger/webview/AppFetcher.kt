// The adapter's own fetcher (027): a download's bytes, fetched in the app's process.
//
// The system's DownloadManager fetched them before. It is handed the cookies for an address,
// and sends them on to every site that address sends it to: one site's cookies, given to
// another. It cannot be told not to. So the adapter fetches itself, one address at a time:
// each address is asked with the login set's cookies for that address and no other's, and
// whoever sends on is followed by hand.
//
// It fetches an address again, as the system's downloader did: so it gets what an address and
// its cookies get, and not what only the page could (a `blob:`, the answer to a posted form).
package com.tigercodes.webtiger.webview

import android.content.ContentResolver
import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Environment
import android.provider.MediaStore
import android.webkit.CookieManager
import androidx.webkit.ProfileStore
import androidx.webkit.WebViewFeature
import java.io.Closeable
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executor
import java.util.concurrent.Executors
import java.util.concurrent.RejectedExecutionException
import java.util.concurrent.atomic.AtomicLong

/** Where downloads are saved. */
enum class DownloadPlace {
    /** The device's Downloads, which every app and the person see. Only the app itself, started for real, saves there. */
    DEVICE,

    /** The app's own folder for downloads: what a test saves into, and fake mode. */
    APP,
}

/** One site's answer to being asked for a file. Closed by whoever was given it. */
internal class Reply(
    val status: Int,
    /** Where it sends whoever asked, when it sends them on. */
    val location: String? = null,
    /** Every cookie the answer sets, each as the site wrote it. */
    val cookies: List<String> = emptyList(),
    /** How many bytes it says are coming. Less than zero when it did not say. */
    val length: Long = -1,
    /** The file's bytes, when the answer is the file. */
    val body: InputStream? = null,
    private val end: () -> Unit = {},
) : Closeable {
    override fun close() {
        try {
            body?.close()
        } catch (_: IOException) {
            // It is being let go of: how it ends is nothing to anybody.
        }
        end()
    }
}

/** A login set's cookies: what an address is sent, and what a site sets. */
internal interface Jar {
    /** The cookies for an address, as a request's header carries them. Null for none. */
    fun cookiesFor(address: String): String?

    /** A site at `address` set a cookie. Whose it may be is the cookie store's to say. */
    fun keep(address: String, cookie: String)
}

/** Where a download's bytes go. */
internal interface Sink {
    @Throws(IOException::class)
    fun write(bytes: ByteArray, count: Int)

    /** The file is whole. The name it has: the one asked for, or another when that was taken. */
    @Throws(IOException::class)
    fun finish(): String

    /** The file is not wanted: nothing of it is left. */
    fun discard()
}

/** How a fetch ended. */
internal sealed interface Fetched {
    data class Whole(val fileName: String) : Fetched

    /** One of the contract's codes: "download.network", "download.disk" or "download.failed". */
    data class Failed(val code: String) : Fetched

    /** It was stopped, and nothing is left of it. */
    data object Stopped : Fetched
}

/** What a fetch needs of the world around it. */
internal class Fetching(
    /** The cookies the login set has for an address. */
    val cookiesFor: (address: String) -> String?,
    /** A site at an address set a cookie. */
    val keep: (address: String, cookie: String) -> Unit,
    /** Asks one address, with those cookies, following nobody. Null when nobody answered. */
    val ask: (address: String, cookies: String?) -> Reply?,
    /** Opens where the bytes go, once a site has answered with the file. Null when it cannot be. */
    val sink: () -> Sink?,
    /** The fetch has been told to stop. */
    val stopped: () -> Boolean = { false },
    /** How far it has got: bytes that have come, of how many (less than zero when not said). */
    val progress: (received: Long, expected: Long) -> Unit = { _, _ -> },
)

/** The answers that send whoever asked on to another address. */
private val SENDS_ON = setOf(301, 302, 303, 307, 308)

/**
 * Fetches the file at `url` (027). Each address on the way is asked with its own cookies and
 * nobody else's; a cookie a site sets is kept for that site, before the next address is asked;
 * whoever sends on is followed by hand, [MOST_HOPS] times at the most and only to an address of
 * the web. The file is the first answer that sends nowhere and says all is well.
 */
internal fun fetch(url: String, with: Fetching): Fetched {
    var at = url
    repeat(MOST_HOPS + 1) {
        if (with.stopped()) return Fetched.Stopped
        val reply = with.ask(at, with.cookiesFor(at)) ?: return Fetched.Failed("download.network")
        reply.use {
            // Kept for the site that set them: the cookie store says whose a cookie may be.
            reply.cookies.forEach { with.keep(at, it) }
            if (reply.status !in SENDS_ON) {
                return if (reply.status in 200..299 && reply.body != null) save(reply.body, reply.length, with) else Fetched.Failed("download.network")
            }
            // Sent on: the next address is a request of its own, with its own cookies.
            val next = reply.location?.let { resolved(at, it) }
            if (next == null || !isOfTheWeb(next)) return Fetched.Failed("download.network")
            at = next
        }
    }
    // Sent on for ever.
    return Fetched.Failed("download.network")
}

/** Moves a file's bytes to where they go. A file that is not whole is not left behind. */
private fun save(body: InputStream, length: Long, with: Fetching): Fetched {
    val sink = with.sink() ?: return Fetched.Failed("download.disk")
    val buffer = ByteArray(BUFFER_BYTES)
    var received = 0L
    with.progress(received, length)
    while (true) {
        if (with.stopped()) {
            sink.discard()
            return Fetched.Stopped
        }
        val count =
            try {
                body.read(buffer)
            } catch (_: IOException) {
                sink.discard()
                return Fetched.Failed("download.network")
            }
        if (count < 0) break
        try {
            sink.write(buffer, count)
        } catch (_: IOException) {
            sink.discard()
            return Fetched.Failed("download.disk")
        }
        received += count
        with.progress(received, length)
    }
    // Fewer bytes than the site said: the connection ended early, and the file is not whole.
    if (length in 0 until Long.MAX_VALUE && received != length) {
        sink.discard()
        return Fetched.Failed("download.network")
    }
    return try {
        Fetched.Whole(sink.finish())
    } catch (_: IOException) {
        sink.discard()
        Fetched.Failed("download.disk")
    }
}

private const val BUFFER_BYTES = 64 * 1024

/**
 * The real fetcher. `jars` gives a login set's cookies, and is asked on the main thread;
 * `sinks` opens the file a download is saved in, under the name asked for; `ask` asks one
 * address. The fetching itself is done on `worker`.
 */
internal class AppFetcher(
    private val jars: (loginSet: String) -> Jar?,
    private val sinks: (name: String) -> Sink?,
    private val ask: (address: String, userAgent: String, cookies: String?) -> Reply? = ::askTheWeb,
    private val worker: Executor = Executors.newCachedThreadPool(),
) : Fetcher {
    /** An engine's fetcher on this device: the login sets' own cookie stores, and the place for downloads. */
    constructor(context: Context, place: DownloadPlace) : this(
        jars = ::profileJar,
        sinks = { name ->
            when (place) {
                DownloadPlace.DEVICE -> MediaStoreSink.open(context.contentResolver, name)
                DownloadPlace.APP -> context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)?.let { FileSink.open(it, name) }
            }
        },
    )

    private class Fetch {
        @Volatile var status: FetchStatus = FetchStatus.Coming(0, 0)

        @Volatile var stopped = false
    }

    private val fetches = ConcurrentHashMap<Long, Fetch>()
    private val handles = AtomicLong()

    override fun start(offer: DownloadOffer, loginSet: String, name: String): Long? {
        if (!isOfTheWeb(offer.url)) return null
        val jar = jars(loginSet)
        // The name is the core's and has no folder in it; it is held to that here all the same.
        val file = name.substringAfterLast('/').substringAfterLast('\\').ifEmpty { "download" }
        val fetch = Fetch()
        val handle = handles.incrementAndGet()
        fetches[handle] = fetch
        return try {
            worker.execute { run(fetch, offer, jar, file) }
            handle
        } catch (_: RejectedExecutionException) {
            fetches.remove(handle)
            null
        }
    }

    private fun run(fetch: Fetch, offer: DownloadOffer, jar: Jar?, file: String) {
        val ended =
            try {
                fetch(
                    offer.url,
                    Fetching(
                        cookiesFor = { address -> jar?.cookiesFor(address) },
                        keep = { address, cookie -> jar?.keep(address, cookie) },
                        ask = { address, cookies -> ask(address, offer.userAgent, cookies) },
                        sink = { sinks(file) },
                        stopped = { fetch.stopped },
                        progress = { received, expected -> fetch.status = FetchStatus.Coming(received, expected) },
                    ),
                )
            } catch (_: RuntimeException) {
                // Whatever went wrong that nobody foresaw: the download failed, and the app goes on.
                Fetched.Failed("download.failed")
            }
        when (ended) {
            is Fetched.Whole -> fetch.status = FetchStatus.Finished(ended.fileName)
            is Fetched.Failed -> fetch.status = FetchStatus.Failed(ended.code)
            // Stopped: nobody asks after it again.
            Fetched.Stopped -> Unit
        }
    }

    override fun stop(handle: Long) {
        fetches.remove(handle)?.stopped = true
    }

    override fun status(handle: Long): FetchStatus? {
        val status = fetches[handle]?.status ?: return null
        // What has ended is told once, and then forgotten.
        if (status !is FetchStatus.Coming) fetches.remove(handle)
        return status
    }
}

/** How long a site is waited for: to answer, and for more of a file. */
private const val CONNECT_PATIENCE_MS = 15_000
private const val READ_PATIENCE_MS = 30_000

/**
 * Asks one address for its file, over the device's own connections, with the cookies it is
 * given and nothing else of anybody's: no sign-in, and no address it was sent from. It follows
 * nobody. Null when nobody answered.
 */
internal fun askTheWeb(address: String, userAgent: String, cookies: String?): Reply? {
    val connection =
        try {
            URL(address).openConnection() as? HttpURLConnection
        } catch (_: IOException) {
            null
        } ?: return null
    return try {
        connection.requestMethod = "GET"
        // Followed by hand: what is sent to one site is not sent to the next.
        connection.instanceFollowRedirects = false
        connection.connectTimeout = CONNECT_PATIENCE_MS
        connection.readTimeout = READ_PATIENCE_MS
        // The bytes as the site has them, and their number as it says it.
        connection.setRequestProperty("Accept-Encoding", "identity")
        if (userAgent.isNotEmpty()) connection.setRequestProperty("User-Agent", userAgent)
        if (!cookies.isNullOrEmpty()) connection.setRequestProperty("Cookie", cookies)
        val status = connection.responseCode
        Reply(
            status = status,
            location = connection.getHeaderField("Location"),
            cookies = connection.headerFields.filterKeys { it.equals("Set-Cookie", ignoreCase = true) }.values.flatten(),
            length = connection.contentLengthLong,
            body = if (status in 200..299) connection.inputStream else null,
            end = connection::disconnect,
        )
    } catch (_: IOException) {
        connection.disconnect()
        null
    } catch (_: RuntimeException) {
        connection.disconnect()
        null
    }
}

/** A login set's cookies in the WebView's own store: the profile's, or the one store of a WebView with no profiles. Asked on the main thread. */
internal fun profileJar(loginSet: String): Jar? {
    val store =
        try {
            if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                ProfileStore.getInstance().getProfile(loginSet)?.cookieManager
            } else {
                CookieManager.getInstance()
            }
        } catch (_: RuntimeException) {
            null
        } ?: return null
    return object : Jar {
        override fun cookiesFor(address: String): String? =
            try {
                store.getCookie(address)
            } catch (_: RuntimeException) {
                null
            }

        override fun keep(address: String, cookie: String) {
            try {
                store.setCookie(address, cookie)
            } catch (_: RuntimeException) {
                // A cookie the store will not have is not kept.
            }
        }
    }
}

/** A file in a folder of the app's own. */
internal class FileSink private constructor(private val file: File, private val out: OutputStream) : Sink {
    override fun write(bytes: ByteArray, count: Int) = out.write(bytes, 0, count)

    override fun finish(): String {
        out.close()
        return file.name
    }

    override fun discard() {
        try {
            out.close()
        } catch (_: IOException) {
            // It is being removed.
        }
        file.delete()
    }

    companion object {
        /** A new file called `name` in `folder`, or with a number after its name when that is taken. Null when none can be made. */
        fun open(folder: File, name: String): FileSink? =
            try {
                folder.mkdirs()
                val stem = name.substringBeforeLast('.', name)
                val ending = if ('.' in name) "." + name.substringAfterLast('.') else ""
                generateSequence(0) { it + 1 }
                    .take(MOST_NAMES)
                    .map { File(folder, if (it == 0) name else "$stem-$it$ending") }
                    // Made here and now, so that no two downloads take one name.
                    .firstOrNull { it.createNewFile() }
                    ?.let { FileSink(it, it.outputStream()) }
            } catch (_: IOException) {
                null
            } catch (_: SecurityException) {
                null
            }

        /** How many names are tried for a file whose name is taken. */
        private const val MOST_NAMES = 1000
    }
}

/**
 * A file in the device's Downloads, through the system's MediaStore: it needs no permission,
 * and the file is the app's own to write. Hidden from other apps until it is whole. The system
 * gives a number to a name that is taken.
 */
internal class MediaStoreSink private constructor(private val resolver: ContentResolver, private val row: Uri, private val out: OutputStream, private val name: String) : Sink {
    override fun write(bytes: ByteArray, count: Int) = out.write(bytes, 0, count)

    override fun finish(): String {
        out.close()
        resolver.update(row, ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) }, null, null)
        // What the system called it: the name asked for, or another when that was taken.
        return resolver.query(row, arrayOf(MediaStore.MediaColumns.DISPLAY_NAME), null, null, null)?.use { found ->
            if (found.moveToFirst()) found.getString(0) else null
        } ?: name
    }

    override fun discard() {
        try {
            out.close()
        } catch (_: IOException) {
            // It is being removed.
        }
        try {
            resolver.delete(row, null, null)
        } catch (_: RuntimeException) {
            // The system will not: there is no more the app can do about it.
        }
    }

    companion object {
        fun open(resolver: ContentResolver, name: String): MediaStoreSink? =
            try {
                val values =
                    ContentValues().apply {
                        put(MediaStore.MediaColumns.DISPLAY_NAME, name)
                        put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
                        put(MediaStore.MediaColumns.IS_PENDING, 1)
                    }
                val row = resolver.insert(MediaStore.Downloads.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY), values)
                val out = row?.let { resolver.openOutputStream(it) }
                if (row != null && out == null) resolver.delete(row, null, null)
                if (row != null && out != null) MediaStoreSink(resolver, row, out, name) else null
            } catch (_: IOException) {
                null
            } catch (_: RuntimeException) {
                // A place the system will not let the app write to.
                null
            }
    }
}
