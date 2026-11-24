// What the system says of its memory, passed on to the core (055). The shell decides nothing:
// it says which of two words it heard, short or very short, and the core puts to sleep the
// tabs that can be.
package com.tigercodes.webtiger.ui

import android.app.ActivityManager
import android.content.ComponentCallbacks2
import android.content.Context
import android.content.res.Configuration
import android.os.Build
import com.tigercodes.webtiger.core.v1.MemoryLevel
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Listens to the system for word that it is short of memory, and says so to the core.
 *
 * Android has two ways, by its version. Before Android 14 the system tells a running app: its
 * "running low" is short, its "running critical" is very short, and so is its "low memory",
 * which comes when every process in the background has been ended. From Android 14 it tells a
 * running app none of these (the system's documents: "not notified … since API level 34",
 * "never called"). What is left there is a question: the system says, when asked, whether it
 * considers itself low on memory. So from Android 14 the watch [asks], once every [every]
 * while the app is in front, and says short each time the answer is yes.
 *
 * Registered with the application as its component callbacks. Not in fake mode: a suite's run
 * must not go by how busy the device it runs on is.
 */
class MemoryWatch(
    private val scope: CoroutineScope,
    /** Whether the system is to be asked: it tells a running app nothing itself (Android 14 on). */
    private val asks: Boolean,
    /** Whether the system considers itself low on memory now. */
    private val isLow: () -> Boolean,
    private val every: Duration = ASK_EVERY,
    /** Says to the core that the system is short of memory, so short. */
    private val say: (MemoryLevel) -> Unit,
) : ComponentCallbacks2 {
    private var asking: Job? = null

    /**
     * The app has come in front, or has left it. While it is in front, and the system is one
     * that must be asked, it is asked every so often; out of sight, nothing is asked: the app
     * is not woken for it.
     */
    fun inFront(front: Boolean) {
        if (!front) {
            asking?.cancel()
            asking = null
            return
        }
        if (!asks || asking != null) return
        asking =
            scope.launch {
                while (true) {
                    delay(every)
                    if (isLow()) say(MemoryLevel.MEMORY_LEVEL_SHORT)
                }
            }
    }

    override fun onTrimMemory(level: Int) {
        level(level)?.let(say)
    }

    /**
     * Before Android 14: every process in the background has been ended, and the ones in front
     * are next. The last word there is.
     */
    @Deprecated("The system calls it before Android 14 only")
    override fun onLowMemory() {
        say(MemoryLevel.MEMORY_LEVEL_VERY_SHORT)
    }

    override fun onConfigurationChanged(newConfig: Configuration) = Unit

    companion object {
        /** How often the system is asked: as often as the core looks for tabs to put to sleep (053). */
        val ASK_EVERY: Duration = 1.minutes

        /** The first Android that tells a running app nothing of its memory: 14. */
        const val TELLS_NOTHING_FROM = Build.VERSION_CODES.UPSIDE_DOWN_CAKE

        /**
         * What the system told a running app, as the contract's word: "running low" is short,
         * "running critical" is very short. Anything else is no word for the core: that the
         * app's window is hidden, or that its process is among those kept in the background,
         * says nothing of how short the system is.
         */
        // The two levels are deprecated because Android 14 and later do not send them. Before
        // that, they are what the system says.
        @Suppress("DEPRECATION")
        fun level(trim: Int): MemoryLevel? =
            when (trim) {
                ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW -> MemoryLevel.MEMORY_LEVEL_SHORT
                ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL -> MemoryLevel.MEMORY_LEVEL_VERY_SHORT
                else -> null
            }

        /** Whether the system considers itself low on memory, as it answers when asked. */
        fun isLow(context: Context): Boolean {
            val manager = context.getSystemService(ActivityManager::class.java) ?: return false
            return ActivityManager.MemoryInfo().also(manager::getMemoryInfo).lowMemory
        }

        /** The watch of the app on this device: it asks where the device's Android tells nothing. */
        fun of(context: Context, scope: CoroutineScope, say: (MemoryLevel) -> Unit): MemoryWatch {
            val application = context.applicationContext
            return MemoryWatch(
                scope = scope,
                asks = Build.VERSION.SDK_INT >= TELLS_NOTHING_FROM,
                isLow = { isLow(application) },
                say = say,
            )
        }
    }
}
