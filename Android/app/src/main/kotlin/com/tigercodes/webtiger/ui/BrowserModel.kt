// A window's model: what the core last said about it, and the user's intents sent back.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.TabCommand
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot

/** One window's state, for Compose to draw. Used on the main thread. */
class BrowserModel(private val session: BrowserSession) {
    /** The tab as the core last published it. */
    var tab by mutableStateOf(TabView())
        private set

    /** What is in the address field. While the field is not being edited it follows the core. */
    var addressText by mutableStateOf("")

    private var editing by mutableStateOf(false)

    /** The address field has the keyboard: what the user types is not overwritten. */
    var isEditingAddress: Boolean
        get() = editing
        set(value) {
            editing = value
            // Editing starts from the whole address; at rest the field shows the short form.
            addressText = if (value && address.url.isNotEmpty()) address.url else address.display
        }

    private var window: OpenedWindow? = null
    private var opening = false

    /** What the address field is told to show. Never null here: an empty tab has an empty one. */
    val address: AddressView get() = tab.address ?: AddressView()

    /** Opens the window in the core. Call once, when the window appears. */
    suspend fun open() {
        if (window != null || opening) return
        opening = true
        try {
            window = session.openWindow(::apply)
        } catch (_: CoreError) {
            // The core would not open a window: the tab stays empty.
        } finally {
            opening = false
        }
    }

    fun close() {
        val closing = window ?: return
        window = null
        session.closeWindow(closing)
    }

    /** Sends the address field's text to the core, which decides what it is. */
    fun submit() {
        val open = window ?: return
        val text = addressText
        isEditingAddress = false
        session.send(Request(submit_address = SubmitAddress(tab_id = open.tabId, text = text)))
    }

    fun perform(command: NavigationCommand) {
        val open = window ?: return
        session.send(Request(tab_command = TabCommand(tab_id = open.tabId, command = command)))
    }

    val isLoading: Boolean get() = tab.load_state == LoadState.LOAD_STATE_LOADING

    val hasFailed: Boolean get() = tab.load_state == LoadState.LOAD_STATE_FAILED

    val hasPage: Boolean get() = tab.web_view_id != 0L

    private fun apply(snapshot: WindowSnapshot) {
        tab = snapshot.tab ?: TabView()
        if (!editing) addressText = address.display
    }
}
