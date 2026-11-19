// What the core asks of the shell (000 §5.5), sorted by who answers: the System service is the
// shell's own, and everything else is a web engine's.
package com.tigercodes.webtiger.ui

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import com.tigercodes.webtiger.core.HostServices
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse

/**
 * Opens an address with whatever the system has for it: the mail app for a letter, the dialler
 * for a number. True when the system took it; false when nothing on the device opens it.
 *
 * The shell opens only what the core asks it to (`OpenOutside`): which addresses may leave the
 * browser, and for whom, is decided there (007).
 */
fun interface OutsideOpener {
    fun open(url: String): Boolean

    companion object {
        /**
         * Opens nothing, and says it did. Fake mode's: a test suite, or a screenshot run, must
         * not start the mail app of the device it runs on.
         */
        val nothing = OutsideOpener { true }
    }
}

/** The system's own: an activity that says it views the address, in a task of its own. */
class SystemOpener(private val context: Context) : OutsideOpener {
    override fun open(url: String): Boolean {
        val address = Uri.parse(url)
        if (address.scheme.isNullOrEmpty()) return false
        // Only an activity that offers itself to links from the web is started: the address
        // came from a page.
        val intent =
            Intent(Intent.ACTION_VIEW, address)
                .addCategory(Intent.CATEGORY_BROWSABLE)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        return try {
            context.startActivity(intent)
            true
        } catch (_: ActivityNotFoundException) {
            false
        } catch (_: SecurityException) {
            // An activity that takes it, and may not be started from here: as good as none.
            false
        }
    }
}

/** Answers the core's host requests: the System service here, the rest by the engine adapter. */
class ShellHost(private val engine: HostServices, private val opener: OutsideOpener) : HostServices {
    override suspend fun answer(request: HostRequest): HostResponse {
        val open = request.open_outside ?: return engine.answer(request)
        return if (opener.open(open.url)) HostResponse(ack = Ack()) else HostResponse(error = Error(code = "system.no_handler"))
    }
}
