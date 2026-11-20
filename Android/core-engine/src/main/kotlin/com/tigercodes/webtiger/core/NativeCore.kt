// The Rust core behind its C ABI (Core/crates/wt-ffi/include/webtiger_core.h), through the JNI
// glue in the same library (Core/crates/wt-ffi/src/android.rs).
package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.CoreConfig
import com.tigercodes.webtiger.core.v1.CoreStatus
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Envelope
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import java.io.IOException
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.coroutines.CoroutineContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch

/** Receives every envelope the core sends: on the core's own thread, one call at a time. */
fun interface EnvelopeListener {
    /** The bytes are a copy the receiver keeps. What this throws is dropped by the glue. */
    fun onEnvelope(envelope: ByteArray)
}

/** What the C ABI's functions return (`WT_OK` and the rest in webtiger_core.h). */
object Status {
    const val OK = 0
    const val UNKNOWN_CORE = 1
    const val NULL_ARGUMENT = 2
    const val TOO_LARGE = 3
    const val MALFORMED = 4
    const val WRONG_THREAD = 5
    const val PANIC = 6
    const val BAD_ARGUMENT = 7
}

/**
 * The six functions of the C ABI, as the JNI glue exports them: libwebtiger_core.so on Android,
 * the same library built for this machine in the JVM tests.
 */
internal object CoreLibrary {
    const val NAV_MAIN_FRAME = 1
    const val NAV_USER_GESTURE = 2
    const val NAV_ALLOW = 0
    const val NAV_CANCEL = 1
    const val NAV_EXTERNAL = 2
    const val VERDICT_ALLOW = 0
    const val VERDICT_BLOCK = 1

    /** The longest message either way (`WT_MAX_MESSAGE_LEN`). */
    const val MAX_MESSAGE_LEN = 4 * 1024 * 1024

    init {
        System.loadLibrary("webtiger_core")
    }

    /** The contract the library implements: (major shl 16) or minor. */
    @JvmStatic external fun version(): Int

    /**
     * Starts a core from an encoded CoreConfig. Its handle, or 0: the listener has then had
     * CoreStatus FATAL with the code, on this thread, when it was the core that refused.
     */
    @JvmStatic external fun start(config: ByteArray, listener: EnvelopeListener): Long

    /** Hands the core an encoded Envelope. [Status.OK] when it was taken. Any thread. */
    @JvmStatic external fun send(core: Long, envelope: ByteArray): Int

    /**
     * Stops a core and waits for its thread to end: the listener is not called once this has
     * returned [Status.OK]. Called from inside the listener it stops nothing
     * ([Status.WRONG_THREAD]).
     */
    @JvmStatic external fun stop(core: Long): Int

    /** The navigation policy's verdict (`NAV_*`); [NAV_CANCEL] on any failure. Any thread. */
    @JvmStatic external fun navCheck(core: Long, url: String, webViewId: Long, flags: Int): Int

    /** Guard's verdict (`VERDICT_*`); [VERDICT_ALLOW] on any failure. Any thread. */
    @JvmStatic external fun guardCheck(core: Long, url: String, sourceUrl: String, webViewId: Long, resourceType: Int): Int
}

/** The running core. One per process. */
class NativeCore private constructor(
    /** Immutable, so the fast path reads it without a lock. */
    private val handle: Long,
    private val receiver: Receiver,
    host: HostServices,
    hostContext: CoroutineContext,
) : CoreClient {
    private val stopped = AtomicBoolean(false)

    /**
     * One coroutine answers host requests, one after the other, so they are performed in the
     * order the core made them.
     */
    private val answering: Job =
        CoroutineScope(hostContext + SupervisorJob()).launch {
            for ((id, request) in receiver.hostRequests) {
                val response =
                    try {
                        host.answer(request)
                    } catch (cancelled: CancellationException) {
                        throw cancelled
                    } catch (_: Exception) {
                        // A code and nothing else: an exception's words could name an address.
                        HostResponse(error = Error(code = "host.failed"))
                    }
                send(Envelope(id = id, host_response = response))
            }
        }

    override val events: Flow<Event> = receiver.events.receiveAsFlow()

    override suspend fun request(request: Request): Response {
        val answer = CompletableDeferred<Response>()
        val id = receiver.wait(answer) ?: throw CoreError.Stopped()
        val status = send(Envelope(id = id, request = request))
        if (status != Status.OK) {
            receiver.forget(id)
            throw if (status == Status.UNKNOWN_CORE) CoreError.Stopped() else CoreError.NotSent(status)
        }
        try {
            return answer.await()
        } catch (cancelled: CancellationException) {
            // The caller stopped waiting; the answer, when it comes, is for nobody.
            receiver.forget(id)
            throw cancelled
        }
    }

    override fun report(event: EngineEvent) {
        // Id 0: a notification, which the core does not answer.
        send(Envelope(request = Request(engine_event = event)))
    }

    override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict {
        var flags = 0
        if (mainFrame) flags = flags or CoreLibrary.NAV_MAIN_FRAME
        if (userGesture) flags = flags or CoreLibrary.NAV_USER_GESTURE
        return when (CoreLibrary.navCheck(handle, url, webViewId, flags)) {
            CoreLibrary.NAV_ALLOW -> NavigationVerdict.ALLOW
            CoreLibrary.NAV_EXTERNAL -> NavigationVerdict.EXTERNAL
            else -> NavigationVerdict.CANCEL
        }
    }

    override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict =
        when (CoreLibrary.guardCheck(handle, url, sourceUrl, webViewId, resource.code)) {
            CoreLibrary.VERDICT_BLOCK -> RequestVerdict.BLOCK
            else -> RequestVerdict.ALLOW
        }

    override fun stop() {
        if (!stopped.compareAndSet(false, true)) return
        answering.cancel()
        // Returns once the core's thread has ended: the listener is silent from here on.
        if (CoreLibrary.stop(handle) == Status.WRONG_THREAD) {
            // Called from inside the listener, where the core cannot wait for its own thread:
            // another thread stops it.
            Thread({
                CoreLibrary.stop(handle)
                receiver.finish()
            }, "wt-core-stop").start()
            return
        }
        receiver.finish()
    }

    private fun send(envelope: Envelope): Int = CoreLibrary.send(handle, envelope.encode())

    companion object {
        /** The contract this shell was built with (`WT_CONTRACT_*` in webtiger_core.h). */
        const val CONTRACT_MAJOR = 1
        const val CONTRACT_MINOR = 16

        /**
         * Starts the core. [config] needs no contract version: it is filled in here. Host
         * requests are answered by [host] in [hostContext]: the main thread for an engine
         * adapter. Throws [CoreError.StartRefused] with the core's code, or
         * [CoreError.ContractMismatch].
         */
        fun start(config: CoreConfig, host: HostServices, hostContext: CoroutineContext): NativeCore {
            val library = CoreLibrary.version()
            if (library ushr 16 != CONTRACT_MAJOR) throw CoreError.ContractMismatch(library)
            val bytes = config.copy(contract_major = CONTRACT_MAJOR, contract_minor = CONTRACT_MINOR).encode()
            val receiver = Receiver()
            val handle = CoreLibrary.start(bytes, receiver)
            if (handle == 0L) throw CoreError.StartRefused(receiver.fatalCode ?: "start.failed")
            return NativeCore(handle, receiver, host, hostContext)
        }
    }
}

/**
 * What the core delivers to: the requests waiting for an answer, and the channels events and
 * host requests leave through.
 */
private class Receiver : EnvelopeListener {
    private val lock = Any()
    private var nextId = 1L
    private val waiting = HashMap<Long, CompletableDeferred<Response>>()
    private var finished = false

    /** The code of a FATAL status, when one arrived: why a start was refused. */
    @Volatile var fatalCode: String? = null
        private set

    val events = Channel<Event>(Channel.UNLIMITED)
    val hostRequests = Channel<Pair<Long, HostRequest>>(Channel.UNLIMITED)

    /** Registers a request's answer. Its id, or null when the core has stopped. */
    fun wait(answer: CompletableDeferred<Response>): Long? =
        synchronized(lock) {
            if (finished) return null
            val id = nextId++
            waiting[id] = answer
            id
        }

    fun forget(id: Long) {
        synchronized(lock) { waiting.remove(id) }
    }

    override fun onEnvelope(envelope: ByteArray) {
        val decoded =
            try {
                Envelope.ADAPTER.decode(envelope)
            } catch (_: IOException) {
                return
            }
        val response = decoded.response
        val event = decoded.event
        val hostRequest = decoded.host_request
        when {
            response != null -> {
                val answer = synchronized(lock) { waiting.remove(decoded.id) } ?: return
                val error = response.error
                when {
                    error != null -> answer.completeExceptionally(CoreError.Refused(error.code, error.field_))
                    response.hasKind() -> answer.complete(response)
                    else -> answer.completeExceptionally(CoreError.Refused("response.empty", ""))
                }
            }
            event != null -> {
                val status = event.status
                if (status != null && status.state == CoreStatus.State.STATE_FATAL) fatalCode = status.code
                events.trySend(event)
            }
            hostRequest != null -> hostRequests.trySend(decoded.id to hostRequest)
        }
    }

    /** The core has stopped: nothing more will arrive. */
    fun finish() {
        val unanswered =
            synchronized(lock) {
                finished = true
                waiting.values.toList().also { waiting.clear() }
            }
        for (answer in unanswered) answer.completeExceptionally(CoreError.Stopped())
        events.close()
        hostRequests.close()
    }
}

/** Whether a response says anything: a response of a kind this build does not know says nothing. */
/**
 * Whether an answer is one of the kinds this shell knows. Each kind the contract gains is named
 * here: one that is not was taken for no answer at all, which is how a new tab's answer, and
 * then a page of history, came back as "response.empty" (found by the History list, on a
 * device).
 */
private fun Response.hasKind(): Boolean = listOf(ack, pong, subscribed, window_opened, tab_opened, history_page, address_suggestions, downloads_page, permissions_page, search_settings).any { it != null }
