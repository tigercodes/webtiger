// The Permissions list's model (014): what the sites of a window's login set were allowed to
// use, as the core says, and taking it back. The shell keeps no list of its own: it asks.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ForgetPermissions
import com.tigercodes.webtiger.core.v1.QueryPermissions
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SitePermissions
import kotlinx.coroutines.launch

/** What the sites of a window's Space were allowed. Used on the main thread, as the window's model is. */
class PermissionsModel(private val session: BrowserSession, private val windowId: Long) {
    /** The sites that were allowed something, in the order the core gives them: by name. */
    var sites by mutableStateOf<List<SitePermissions>>(emptyList())
        private set

    /**
     * The core has answered at least once. Until then the list says nothing at all, rather than
     * that no site was allowed anything.
     */
    var loaded by mutableStateOf(false)
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var asked = 0

    /** Asks for the list again. */
    fun reload() {
        asked += 1
        val question = asked
        // Started after whatever was sent before it, on the same thread: a question that follows
        // a taking back is answered with what is left.
        session.scope.launch {
            val page =
                try {
                    session.core.request(Request(query_permissions = QueryPermissions(window_id = windowId))).permissions_page
                } catch (_: CoreError) {
                    null
                } ?: return@launch
            if (question != asked) return@launch
            sites = page.sites
            loaded = true
        }
    }

    /**
     * Takes back what one site was allowed. It leaves the list at once, and will be asked about
     * the next time one of its pages wants what it had; nothing that is running is stopped.
     */
    fun forget(site: SitePermissions) {
        session.send(Request(forget_permissions = ForgetPermissions(window_id = windowId, origin = site.origin)))
        sites = sites.filter { it.origin != site.origin }
    }

    /** Takes back what every site was allowed. */
    fun forgetEverything() {
        session.send(Request(forget_permissions = ForgetPermissions(window_id = windowId, everything = true)))
        reload()
    }
}
