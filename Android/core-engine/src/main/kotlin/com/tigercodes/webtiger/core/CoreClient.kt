// The core as Kotlin sees it (003 §4): requests in, answers and snapshots out, host services
// answered. The shell holds no browser logic; it asks here.
package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import kotlinx.coroutines.flow.Flow

/** Why a call to the core failed. Its message is a code, never an address or typed text. */
sealed class CoreError(message: String) : Exception(message) {
    /**
     * The core answered with an error. [code] is stable ("tab.unknown"); the shell turns it into
     * words.
     */
    data class Refused(val code: String, val field: String) : CoreError(code)

    /** The core would not start, with the code its CoreStatus carried ("start.data_dir"). */
    data class StartRefused(val code: String) : CoreError(code)

    /** The library speaks another major version of the contract. */
    data class ContractMismatch(val library: Int) : CoreError("contract.mismatch")

    /** The library did not take the message, and returned this status instead. */
    data class NotSent(val status: Int) : CoreError("core.not_sent")

    /** The core has stopped. */
    class Stopped : CoreError("core.stopped") {
        override fun equals(other: Any?): Boolean = other is Stopped

        override fun hashCode(): Int = 1
    }
}

/** What the navigation policy says about a navigation a web view is about to make. */
enum class NavigationVerdict {
    ALLOW,
    CANCEL,

    /** Not a page: refuse it in the web view and hand the address to the system. */
    EXTERNAL,
}

/** What Guard says about a request a page is about to make. */
enum class RequestVerdict {
    ALLOW,
    BLOCK,
}

/** Whether a frame has its site's own storage, in the page it is in (071). */
enum class StorageVerdict {
    /** The frame has its site's storage, as the engine gives it. */
    OWN,

    /** The frame is kept from its site's storage: what it keeps goes with its document. */
    APART,
}

/** What a request is for, as far as the engine says (`WT_RESOURCE_*` in webtiger_core.h). */
enum class ResourceType(val code: Int) {
    OTHER(0),
    DOCUMENT(1),
    SUBDOCUMENT(2),
    SCRIPT(3),
    STYLESHEET(4),
    IMAGE(5),
    FONT(6),
    MEDIA(7),
    XHR(8),
    WEBSOCKET(9),
    PING(10),
}

/** What the core asks the platform for (000 §5.5). The engine adapter answers. */
interface HostServices {
    /**
     * Answers one host request. Requests arrive, and are answered, in the order the core made
     * them: a web view's creation always comes before its first load.
     */
    suspend fun answer(request: HostRequest): HostResponse
}

/** The core. [NativeCore] is the real one; [ScriptedCore] stands in for it in tests. */
interface CoreClient {
    /**
     * Sends a request and waits for its answer. Throws [CoreError.Refused] when the answer is an
     * error, so what is returned is never one.
     */
    suspend fun request(request: Request): Response

    /** Reports what a web view did. A notification: nothing comes back. */
    fun report(event: EngineEvent)

    /** Everything the core says unasked, in order. One collector. */
    val events: Flow<Event>

    /**
     * The fast path: whether a web view may make a navigation. Synchronous, and callable from
     * any thread.
     */
    fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict

    /**
     * The fast path: whether a page may make a request. Synchronous, and callable from any
     * thread, the engine's own among them. [sourceUrl] is the address of the page that asks.
     */
    fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict

    /**
     * The fast path: whether a frame has its site's own storage (071), asked as its document
     * starts by an engine that does not keep a third party's storage apart itself.
     * Synchronous, and callable from any thread. [origin] is the frame's origin and
     * [topOrigin] that of the top frame above it, both as the frame's own document has them;
     * [pageUrl] is the address of the page as the shell knows it.
     */
    fun storageVerdict(origin: String, topOrigin: String, pageUrl: String, webViewId: Long): StorageVerdict

    /** Stops the core. Requests still waiting throw [CoreError.Stopped]. */
    fun stop()
}

/** Sends a request whose answer says nothing more than that it was done. */
suspend fun CoreClient.perform(request: Request) {
    request(request)
}
