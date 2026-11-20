// A page asks to use the camera, the microphone or the device's location (014), on the System
// WebView. The adapter reports the request and waits: the core says how the page is answered.
// It decides nothing about the site. What it does see to is the system's own permission, which
// is the app's and not the site's: a page the core lets have the camera gets it only if the
// system lets the app.
package com.tigercodes.webtiger.webview

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.PermissionRequested
import com.tigercodes.webtiger.core.v1.PermissionWithdrawn

/** What the system lets the app itself use, and how it is asked for more. */
interface SystemPermissions {
    /** Whether the app holds what the system wants it to hold for each of these. */
    fun holds(kinds: List<PermissionKind>): Boolean

    /**
     * Sees to it that the app holds what these need, asking the system if it may be asked, and
     * says whether it then does. The system asks the person with a dialog of its own.
     */
    fun ask(kinds: List<PermissionKind>, done: (Boolean) -> Unit)
}

/** For an engine that is given none: the app holds nothing, and nobody is asked. */
internal object NoSystemPermissions : SystemPermissions {
    override fun holds(kinds: List<PermissionKind>): Boolean = false

    override fun ask(kinds: List<PermissionKind>, done: (Boolean) -> Unit) = done(false)
}

/**
 * The app's own permissions. Until it is given a way to ask ([asker]) it uses what the app
 * already holds and asks nobody: so it is for every engine a test makes and for fake mode,
 * where nothing may bring the system's dialog up. The app's window, started for real, gives it
 * the way to ask.
 */
class AppPermissions internal constructor(private val held: (String) -> Boolean) : SystemPermissions {
    constructor(context: Context) : this({ name ->
        context.applicationContext.checkSelfPermission(name) == PackageManager.PERMISSION_GRANTED
    })

    /**
     * How the system is asked for permissions, by name; it calls back when the system has
     * answered, whatever the answer. Null while there is no window to ask from: nobody is asked.
     */
    var asker: ((Array<String>, () -> Unit) -> Unit)? = null

    private class Question(val kinds: List<PermissionKind>, val done: (Boolean) -> Unit)

    /** What waits for the system: it is asked one thing at a time, in order. */
    private val waiting = ArrayDeque<Question>()
    private var asking = false

    override fun holds(kinds: List<PermissionKind>): Boolean = kinds.all { kind -> names(kind).let { it.isNotEmpty() && it.any(held) } }

    override fun ask(kinds: List<PermissionKind>, done: (Boolean) -> Unit) {
        waiting.addLast(Question(kinds, done))
        next()
    }

    private fun next() {
        while (!asking) {
            val question = waiting.removeFirstOrNull() ?: return
            val asker = asker
            // With nobody to ask, and for what the system has granted since it was asked for
            // (a question before this one wanted the same), the answer is what the app holds.
            if (asker == null || holds(question.kinds)) {
                question.done(holds(question.kinds))
                continue
            }
            asking = true
            asker(question.kinds.flatMap(::names).distinct().toTypedArray()) {
                asking = false
                question.done(holds(question.kinds))
                next()
            }
        }
    }

    internal companion object {
        /**
         * The system's permissions a kind needs: the app must hold one of them. The location is
         * asked for both ways, as the system requires, and held when either was granted: a
         * person may give only where they roughly are.
         */
        fun names(kind: PermissionKind): List<String> =
            when (kind) {
                PermissionKind.PERMISSION_KIND_CAMERA -> listOf(Manifest.permission.CAMERA)
                PermissionKind.PERMISSION_KIND_MICROPHONE -> listOf(Manifest.permission.RECORD_AUDIO)
                PermissionKind.PERMISSION_KIND_LOCATION -> listOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION)
                else -> emptyList()
            }
    }
}

/** The requests of one engine's pages that wait for the core's word. Used on the main thread, as the engine is. */
internal class Permissions(private val system: SystemPermissions, private val report: (EngineEvent) -> Unit) {
    private class Waiting(val webViewId: Long, val kinds: List<PermissionKind>, val answer: (Boolean) -> Unit)

    /**
     * How many requests this engine has named. Each has the next number, and none is used twice:
     * the ids are the shell's own (engine.proto, `PermissionRequested`).
     */
    private var named = 0L
    private val waiting = HashMap<Long, Waiting>()

    /**
     * A page of `webViewId` asks, as `origin`, for these. `answer` tells the page yes or no,
     * once. Nothing is answered until the core has said. The adapter's name for the request.
     */
    fun asked(webViewId: Long, origin: String, kinds: List<PermissionKind>, answer: (Boolean) -> Unit): Long {
        named += 1
        waiting[named] = Waiting(webViewId, kinds, answer)
        report(EngineEvent(web_view_id = webViewId, permission_requested = PermissionRequested(request_id = named, origin = origin, kinds = kinds)))
        return named
    }

    /** The page no longer waits for an answer to a request: the core is told, and nothing is answered. */
    fun withdrawn(id: Long) {
        val request = waiting.remove(id) ?: return
        report(EngineEvent(web_view_id = request.webViewId, permission_withdrawn = PermissionWithdrawn(request_id = id)))
    }

    /**
     * The core's word on a request. "No" is told the page. "Yes" is the core's about the site:
     * the page is told yes when the system lets the app, which is asked if it has not said.
     * A request that no longer waits has no page to answer.
     */
    fun answer(id: Long, granted: Boolean) {
        val request = waiting.remove(id) ?: return
        when {
            !granted -> request.answer(false)
            system.holds(request.kinds) -> request.answer(true)
            else -> system.ask(request.kinds) { held -> request.answer(held) }
        }
    }

    /** A web view is going: what its pages asked is refused, and nothing more is said of it. */
    fun dropped(webViewId: Long) {
        val gone = waiting.filterValues { it.webViewId == webViewId }
        for ((id, request) in gone) {
            waiting.remove(id)
            request.answer(false)
        }
    }
}
