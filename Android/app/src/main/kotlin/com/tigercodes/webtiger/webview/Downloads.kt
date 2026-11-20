// Downloads (013 §6), on the System WebView. The WebView does not download: it says a download
// was asked for, and whoever embeds it must fetch the address again. The adapter reports the
// download, waits for the core's word, has the file fetched under the name the core made, and
// says how it goes. It decides nothing, and never writes a name a page gave it.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.DownloadFailed
import com.tigercodes.webtiger.core.v1.DownloadFinished
import com.tigercodes.webtiger.core.v1.DownloadProgress
import com.tigercodes.webtiger.core.v1.DownloadRequested
import com.tigercodes.webtiger.core.v1.EngineEvent

/** What a page offered to be downloaded, as the WebView said it. */
internal data class DownloadOffer(
    val url: String,
    /** What the WebView's requests say they are, so the fetch says the same. */
    val userAgent: String,
    /** What the site suggests the file is called. Never a name to save under: the core makes that. */
    val suggestedName: String,
    val mimeType: String,
    /** How big the site says the file is. Zero or less when it did not say. */
    val length: Long,
    /** The person chose it in a menu (022): no page offered it, and the core asks nobody. */
    val chosen: Boolean = false,
)

/** How a fetch stands. */
internal sealed interface FetchStatus {
    data class Coming(val received: Long, val expected: Long) : FetchStatus

    /** The file is whole, under `fileName`: the name asked for, or the one used when that was taken. */
    data class Finished(val fileName: String) : FetchStatus

    /** One of the contract's codes: "download.network", "download.disk" or "download.failed". */
    data class Failed(val code: String) : FetchStatus
}

/** What fetches a download's bytes. [SystemFetcher] is the real one: the system's own. */
internal interface Fetcher {
    /**
     * Starts fetching an offer's address, with the login set's cookies for it, into a file
     * called `name` in the place for downloads. A handle to ask about the fetch by; null when
     * it could not be started.
     */
    fun start(offer: DownloadOffer, loginSet: String, name: String): Long?

    /** Stops a fetch, and removes what it had saved. */
    fun stop(handle: Long)

    /** How a fetch stands. Null when the fetcher no longer knows of it. */
    fun status(handle: Long): FetchStatus?
}

/** A fetcher for an engine that is given none: it starts nothing. */
internal object NoFetcher : Fetcher {
    override fun start(offer: DownloadOffer, loginSet: String, name: String): Long? = null

    override fun stop(handle: Long) = Unit

    override fun status(handle: Long): FetchStatus? = null
}

/** The downloads of one engine. Used on the main thread, as the engine is. */
internal class Downloads(private val fetcher: Fetcher, private val asker: Asker = NoAsker, private val report: (EngineEvent) -> Unit) {
    private class Asked(val webViewId: Long, val loginSet: String, val offer: DownloadOffer)

    private class Coming(val webViewId: Long, val handle: Long, var said: FetchStatus.Coming? = null)

    /**
     * How many downloads this engine has named. Each has the next number, and none is used
     * twice: the ids are the shell's own (engine.proto, `DownloadRequested`).
     */
    private var named = 0L

    /** Reported to the core, and waiting for its word. */
    private val asked = HashMap<Long, Asked>()
    private val coming = HashMap<Long, Coming>()

    /** Something is being fetched: its progress is worth asking after. */
    val isBusy: Boolean get() = coming.isNotEmpty()

    /**
     * A page of `webViewId`, in `loginSet`, asked for a download; or the person chose a file in
     * a menu (022). Nothing is fetched yet. A file the person chose is only an address: where
     * it is, and what its site says of it, is asked first (FileAsker.kt), and it is offered as
     * that.
     */
    fun offered(webViewId: Long, loginSet: String, offer: DownloadOffer) {
        if (!offer.chosen) {
            offer(webViewId, loginSet, offer)
            return
        }
        asker.ask(offer.url, offer.userAgent, loginSet) { told ->
            offer(webViewId, loginSet, offer.copy(url = told.url, suggestedName = told.suggestedName, mimeType = told.mimeType, length = told.length))
        }
    }

    private fun offer(webViewId: Long, loginSet: String, offer: DownloadOffer) {
        named += 1
        asked[named] = Asked(webViewId, loginSet, offer)
        report(
            EngineEvent(
                web_view_id = webViewId,
                download_requested =
                    DownloadRequested(
                        download_id = named,
                        url = offer.url,
                        suggested_filename = offer.suggestedName,
                        mime_type = offer.mimeType,
                        expected_bytes = offer.length.coerceAtLeast(0),
                        user_asked = offer.chosen,
                    ),
            ),
        )
    }

    /**
     * The core's word: the download starts, saved under `name`. False for a download that is
     * not waiting for one. A fetch the system will not start is a download that failed.
     */
    fun start(id: Long, name: String): Boolean {
        val waiting = asked.remove(id) ?: return false
        val handle = fetcher.start(waiting.offer, waiting.loginSet, name)
        if (handle == null) {
            failed(id, waiting.webViewId, "download.failed")
        } else {
            coming[id] = Coming(waiting.webViewId, handle)
        }
        return true
    }

    /**
     * The core's word: the download is dropped. One that was waiting never starts; one that is
     * coming stops, and what was saved of it goes. Nothing more is said of either.
     */
    fun cancel(id: Long) {
        asked.remove(id)
        coming.remove(id)?.let { fetcher.stop(it.handle) }
    }

    /**
     * Asks how each download that is coming stands, and reports what has changed: how far it
     * has got, that it is whole, or that it failed.
     */
    fun poll() {
        for ((id, download) in coming.toList()) {
            when (val status = fetcher.status(download.handle)) {
                is FetchStatus.Coming ->
                    if (status != download.said) {
                        download.said = status
                        report(
                            EngineEvent(
                                web_view_id = download.webViewId,
                                download_progress =
                                    DownloadProgress(
                                        download_id = id,
                                        received_bytes = status.received.coerceAtLeast(0),
                                        expected_bytes = status.expected.coerceAtLeast(0),
                                    ),
                            ),
                        )
                    }
                is FetchStatus.Finished -> {
                    coming.remove(id)
                    report(EngineEvent(web_view_id = download.webViewId, download_finished = DownloadFinished(download_id = id, file_name = status.fileName)))
                }
                is FetchStatus.Failed -> {
                    coming.remove(id)
                    failed(id, download.webViewId, status.code)
                }
                // The fetcher has forgotten it: somebody removed it from under the app.
                null -> {
                    coming.remove(id)
                    failed(id, download.webViewId, "download.cancelled")
                }
            }
        }
    }

    private fun failed(id: Long, webViewId: Long, code: String) {
        report(EngineEvent(web_view_id = webViewId, download_failed = DownloadFailed(download_id = id, code = code)))
    }
}
