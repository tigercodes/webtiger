// What is awake (056): every tab that has a page loaded, in every window and Space, and why
// each is awake; and the command that puts the others to sleep. The shell keeps no list of its
// own, and decides nothing: it asks.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.AwakeTab
import com.tigercodes.webtiger.core.v1.QueryAwake
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SleepOthers
import kotlinx.coroutines.launch

/** What is awake, as a window is told it. Used on the main thread, as the window's model is. */
class AwakeModel(private val session: BrowserSession, private val windowId: Long) {
    /** The tabs that are awake, in the core's order: this window's first. */
    var tabs by mutableStateOf<List<AwakeTab>>(emptyList())
        private set

    /** How many more are awake in private windows, and are not named here. */
    var privateTabs by mutableStateOf(0)
        private set

    /** How many tabs are asleep. */
    var asleep by mutableStateOf(0)
        private set

    /**
     * The core has answered at least once. Until then the list says nothing at all, rather than
     * that no tab is awake.
     */
    var loaded by mutableStateOf(false)
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var asked = 0

    /**
     * Whether the command would do anything: some tab that is listed is out of sight with
     * nothing to keep it awake. (The core decides which sleep: this only says whether the button
     * is worth pressing.)
     */
    val othersCouldSleep: Boolean
        get() = tabs.any { !it.in_sight && it.keeps_awake.isEmpty() && !it.private_window }

    /** Asks what is awake now. */
    fun reload() {
        asked += 1
        val question = asked
        // Started after whatever was sent before it, on the same thread: a question that follows
        // the command is answered with what the command left awake.
        session.scope.launch {
            val page =
                try {
                    session.core.request(Request(query_awake = QueryAwake(window_id = windowId))).awake_page
                } catch (_: CoreError) {
                    null
                } ?: return@launch
            if (question != asked) return@launch
            tabs = page.tabs
            privateTabs = page.private_tabs
            asleep = page.asleep
            loaded = true
        }
    }

    /** "Sleep all others": the core puts to sleep every tab that can be, and is asked again for what is left. */
    fun sleepOthers() {
        session.send(Request(sleep_others = SleepOthers(window_id = windowId)))
        reload()
    }
}
