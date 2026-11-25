// A core that does what a test tells it to: a screen is tested without the Rust core, by
// scripting the answers and pushing the snapshots it should draw.
package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.Snapshot
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

/**
 * A core that answers every request with what [script] returns; the default answers Ack. A
 * script refuses by throwing [CoreError.Refused].
 */
class ScriptedCore(private val script: (Request) -> Response = { Response(ack = Ack()) }) : CoreClient {
    private val lock = Any()
    private val made = ArrayList<Request>()
    private val reported = ArrayList<EngineEvent>()
    private val navigationVerdicts = HashMap<String, NavigationVerdict>()
    private val requestVerdicts = HashMap<String, RequestVerdict>()
    private val asked = ArrayList<Pair<String, ResourceType>>()
    private var stopped = false
    private val channel = Channel<Event>(Channel.UNLIMITED)

    override val events: Flow<Event> = channel.receiveAsFlow()

    /** Every request made, in order. */
    val requests: List<Request> get() = synchronized(lock) { made.toList() }

    /** Every report made, in order. */
    val reports: List<EngineEvent> get() = synchronized(lock) { reported.toList() }

    /** Every request a page made that was put to Guard, in order: its address, and what it was said to be for. */
    val guarded: List<Pair<String, ResourceType>> get() = synchronized(lock) { asked.toList() }

    val isStopped: Boolean get() = synchronized(lock) { stopped }

    /** Sends the shell an event, as the core would. */
    fun emit(event: Event) {
        channel.trySend(event)
    }

    /** Sends the shell a window's snapshot. */
    fun emit(window: WindowSnapshot, subscription: Long) {
        emit(Event(snapshot = Snapshot(subscription_id = subscription, window = window)))
    }

    /** Sets the navigation policy's answer for an address. Addresses not set are allowed. */
    fun setVerdict(verdict: NavigationVerdict, url: String) {
        synchronized(lock) { navigationVerdicts[url] = verdict }
    }

    /** Sets Guard's answer for a request's address. Addresses not set are allowed. */
    fun setVerdict(verdict: RequestVerdict, url: String) {
        synchronized(lock) { requestVerdicts[url] = verdict }
    }

    override suspend fun request(request: Request): Response {
        val wasStopped =
            synchronized(lock) {
                made.add(request)
                stopped
            }
        if (wasStopped) throw CoreError.Stopped()
        val response = script(request)
        response.error?.let { throw CoreError.Refused(it.code, it.field_) }
        return response
    }

    override fun report(event: EngineEvent) {
        synchronized(lock) { reported.add(event) }
    }

    override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict =
        synchronized(lock) { navigationVerdicts[url] } ?: NavigationVerdict.ALLOW

    override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict =
        synchronized(lock) {
            asked.add(url to resource)
            requestVerdicts[url]
        } ?: RequestVerdict.ALLOW

    override fun stop() {
        synchronized(lock) { stopped = true }
        channel.close()
    }
}
