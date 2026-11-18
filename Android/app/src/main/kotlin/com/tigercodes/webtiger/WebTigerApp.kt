// The app's process: one core, started with the first window.
package com.tigercodes.webtiger

import android.app.Application
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.ui.BrowserSession
import com.tigercodes.webtiger.ui.LaunchOptions

class WebTigerApp : Application() {
    private var started: Result<BrowserSession>? = null

    /**
     * The process's one session, started on first use with what the first window was launched
     * with. A start that failed is not tried again: the window says why.
     */
    fun session(launch: LaunchOptions): Result<BrowserSession> =
        started ?: runCatching { BrowserSession.live(this, launch) }.also { started = it }

    companion object {
        /** A failed start as a code for the window to show: the core's own, or that the core is not there. */
        fun startCode(failure: Throwable): String = (failure as? CoreError)?.message ?: "core.unavailable"
    }
}
