// What a System WebView's callbacks mean, as the contract's events (001 §5.1, 003 §5).
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.AddressChanged
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.HistoryChanged
import com.tigercodes.webtiger.core.v1.NavigationCommitted
import com.tigercodes.webtiger.core.v1.NavigationFailed
import com.tigercodes.webtiger.core.v1.NavigationFinished
import com.tigercodes.webtiger.core.v1.NavigationStarted
import com.tigercodes.webtiger.core.v1.ProgressChanged
import com.tigercodes.webtiger.core.v1.RendererGone
import com.tigercodes.webtiger.core.v1.TitleChanged

/**
 * Turns one web view's callbacks into the events the core expects, in the order it expects
 * them: started, committed, finished or failed. It has no Android in it, so the rules below
 * are tested on the JVM; [WebViewPage] only forwards the engine's callbacks here.
 *
 * The System WebView does not say these things directly, and what it does say depends on its
 * version:
 *
 * - **`onPageStarted` is not the start of a load.** A current WebView calls it when a page
 *   *commits*; an older one calls it when a load the app asked for begins, and again for each
 *   redirect. So a load is taken to have started at whichever comes first of: the adapter's own
 *   load or reload, a main-frame navigation the policy allowed, `onPageStarted`, and progress.
 * - **The commit is `doUpdateVisitedHistory`**, which the engine calls for every main-frame
 *   navigation that committed, a fragment or History API change included. After one of the
 *   start signals it is a new document ([NavigationCommitted]); with none before it, the page
 *   changed its own address ([AddressChanged]). `onPageCommitVisible` is not used: it waits
 *   for a frame to be drawn, which a web view that is not on screen may never do.
 * - **`onPageFinished` is not always called** (a page restored from the back/forward cache
 *   has none) and is called for loads that never committed (stopped, a download). So a load
 *   ends at `onPageFinished` or at progress 100, whichever comes first, and one that ends
 *   without having committed is reported as cancelled.
 * - **A failed load is followed by the engine's own error page**, which commits at the failed
 *   address and finishes like any page. None of that is a new load.
 * - **A refused navigation is followed by progress of its own**, 10 and then 100, or by a
 *   finished page when it was a redirect. Neither is a load.
 *
 * The two rules of engine.proto hold throughout: an address is reported as the page's only
 * from a commit, and a navigation refused on the core's verdict is reported as cancelled.
 */
internal class PageTracker(private val webViewId: Long, private val report: (EngineEvent) -> Unit) {
    /** The core has been told a load began, and not yet how it ended. */
    private var loading = false

    /** A new document is expected: the next commit is its, not the page changing its own address. */
    private var awaitingCommit = false

    /** Where the load in flight is heading, when the engine or the adapter said. */
    private var heading: String? = null

    /** A load to this address ended without committing; should it commit after all, it is a new document. */
    private var lateCommit: String? = null

    /** The load to this address was reported as failed. What the engine does next is its error page. */
    private var failedUrl: String? = null

    /** A navigation to this address was refused; the engine reports its end as a finished page. */
    private var refusedUrl: String? = null

    /** The process drawing the page died, and nothing has been loaded since. */
    private var gone = false

    /** The address last reported as the page's. */
    var pageUrl: String? = null
        private set

    /** What a reload should load, when the engine itself has nothing to reload: the address that failed, or the page a dead renderer was showing. */
    val reloadTarget: String? get() = failedUrl ?: pageUrl.takeIf { gone }

    /** The adapter is about to load an address. */
    fun willLoad(url: String) = begin(url)

    /** The adapter is about to reload the page. With nothing ever committed the engine has nothing to reload, and nothing is said. */
    fun willReload() {
        pageUrl?.let(::begin)
    }

    /**
     * The adapter is about to go back or forward. That may bring a new document or only another
     * state of this one, so the engine says which; but the page a failure covered is left.
     */
    fun willTraverse() {
        failedUrl = null
        lateCommit = null
        refusedUrl = null
    }

    /** The adapter is about to stop the load. The engine may say nothing more about it, so its end is reported here. */
    fun willStop() = end(heading.orEmpty())

    /** `shouldOverrideUrlLoading`: the policy let a main-frame navigation go ahead. */
    fun navigationAllowed(url: String) = begin(url)

    /**
     * `shouldOverrideUrlLoading`: a main-frame navigation was refused on the core's verdict.
     * The engine only reports a page as finished, so the adapter says the navigation ended.
     */
    fun navigationRefused(url: String, isRedirect: Boolean) {
        refusedUrl = url
        val endsTheLoad = loading && awaitingCommit && isRedirect
        if (endsTheLoad) {
            loading = false
            awaitingCommit = false
            heading = null
        }
        emit(EngineEvent(web_view_id = webViewId, navigation_failed = NavigationFailed(code = "net.cancelled", url = url)))
        // The core takes "cancelled" as the end of loading. A load that is still going is one
        // the refused navigation did not belong to, and it goes on.
        if (loading) emit(EngineEvent(web_view_id = webViewId, navigation_started = NavigationStarted(url = heading.orEmpty())))
    }

    /** `onPageStarted`. */
    fun pageStarted(url: String) {
        // The engine announcing the load whose failure has been reported already.
        if (url == failedUrl) return
        begin(url)
    }

    /** `doUpdateVisitedHistory`: a main-frame navigation committed at `url`. */
    fun committed(url: String, canGoBack: Boolean, canGoForward: Boolean) {
        val newDocument = awaitingCommit || url == lateCommit
        awaitingCommit = false
        lateCommit = null
        heading = null
        refusedUrl = null
        if (newDocument) {
            emit(EngineEvent(web_view_id = webViewId, navigation_committed = NavigationCommitted(url = url)))
        } else if (url != pageUrl) {
            emit(EngineEvent(web_view_id = webViewId, address_changed = AddressChanged(url = url)))
        }
        pageUrl = url
        emit(EngineEvent(web_view_id = webViewId, history_changed = HistoryChanged(can_go_back = canGoBack, can_go_forward = canGoForward)))
    }

    /** `onPageFinished`. */
    fun pageFinished(url: String) {
        if (url == refusedUrl) {
            // The end of the navigation that was refused, not of the page.
            refusedUrl = null
            return
        }
        end(url)
    }

    /** `onProgressChanged`, 0 to 100. */
    fun progress(percent: Int) {
        // The engine loading its own error page.
        if (failedUrl != null) return
        if (refusedUrl != null) {
            // A navigation that was refused has progress of its own, 10 and then 100, with no
            // page on its way: not a load, and not the end of one that is going on.
            if (percent >= 100) refusedUrl = null
            if (!loading || percent >= 100) return
        }
        if (percent < 100 && !loading) {
            // A load nothing else announced: back or forward, or a form's submission.
            loading = true
            emit(EngineEvent(web_view_id = webViewId, navigation_started = NavigationStarted(url = heading.orEmpty())))
        }
        if (loading) {
            emit(EngineEvent(web_view_id = webViewId, progress_changed = ProgressChanged(fraction = percent.coerceIn(0, 100) / 100.0)))
        }
        if (percent >= 100) end(heading.orEmpty())
    }

    /** `onReceivedTitle`. */
    fun title(title: String) {
        // The error page's title is the engine's words, not the page's.
        if (failedUrl != null) return
        emit(EngineEvent(web_view_id = webViewId, title_changed = TitleChanged(title = title)))
    }

    /** `onReceivedError` for the main frame, as one of the contract's codes. */
    fun failed(code: String, url: String) {
        // Said already: a certificate failure the engine then reports in its own way.
        if (url == failedUrl) return
        failedUrl = url
        loading = false
        // The error page's commit, if one comes, is a new document's.
        lateCommit = url
        emit(EngineEvent(web_view_id = webViewId, navigation_failed = NavigationFailed(code = code, url = url)))
    }

    /**
     * `onReceivedSslError`, which the adapter always answers by cancelling. Only the page's own
     * load fails by it; a resource of the page that was refused is the page's business.
     */
    fun certificateRefused(url: String) {
        if (loading && awaitingCommit && url == heading) failed("net.tls", url)
    }

    /** `onRenderProcessGone`. The adapter replaces the web view, so its history is gone too. */
    fun rendererGone() {
        loading = false
        awaitingCommit = false
        heading = null
        lateCommit = null
        failedUrl = null
        refusedUrl = null
        gone = true
        emit(EngineEvent(web_view_id = webViewId, renderer_gone = RendererGone()))
        emit(EngineEvent(web_view_id = webViewId, history_changed = HistoryChanged()))
    }

    /** A new document is on its way. */
    private fun begin(url: String) {
        failedUrl = null
        lateCommit = null
        // An address refused before may be allowed now: its page's end is then the page's.
        if (url == refusedUrl) refusedUrl = null
        gone = false
        awaitingCommit = true
        heading = url.ifEmpty { null }
        if (!loading) {
            loading = true
            emit(EngineEvent(web_view_id = webViewId, navigation_started = NavigationStarted(url = url)))
        }
    }

    /** The load is over: finished if its page committed, cancelled if none did. */
    private fun end(url: String) {
        if (!loading) return
        loading = false
        if (awaitingCommit) {
            awaitingCommit = false
            lateCommit = url.ifEmpty { heading }
            heading = null
            emit(EngineEvent(web_view_id = webViewId, navigation_failed = NavigationFailed(code = "net.cancelled", url = url)))
        } else {
            emit(EngineEvent(web_view_id = webViewId, navigation_finished = NavigationFinished()))
        }
    }

    private fun emit(event: EngineEvent) = report(event)
}
