// What the app runs on (054): the core works out from it how many tabs may be alive at once.
// The shell says it once, as it starts; which tabs sleep, and when, is the core's.
package com.tigercodes.webtiger.ui

import android.app.ActivityManager
import android.content.Context
import com.tigercodes.webtiger.core.v1.DeviceKind

/** The device as the app finds it when it starts: a phone or a tablet, and its memory in megabytes. */
data class Device(
    val kind: DeviceKind,
    val megabytes: Int,
) {
    companion object {
        /** The narrower side of a tablet's screen is this many dp or more: the system's own line (`sw600dp`). */
        const val TABLET_FROM_DP = 600

        private const val BYTES_A_MEGABYTE = 1024L * 1024L

        /** A screen whose narrower side is 600dp or more is a tablet's; a narrower one, or one of no known size, a phone's. */
        fun kind(smallestWidthDp: Int): DeviceKind = if (smallestWidthDp >= TABLET_FROM_DP) DeviceKind.DEVICE_KIND_TABLET else DeviceKind.DEVICE_KIND_PHONE

        /** Bytes as whole megabytes: none for a number that is no size, and never more than the contract's field holds. */
        fun megabytes(bytes: Long): Int = (bytes.coerceAtLeast(0) / BYTES_A_MEGABYTE).coerceAtMost(Int.MAX_VALUE.toLong()).toInt()

        /** This device. A foldable is what it was when the app started: opened or shut after, it is not asked again. */
        fun of(context: Context): Device {
            val memory = ActivityManager.MemoryInfo()
            (context.getSystemService(Context.ACTIVITY_SERVICE) as? ActivityManager)?.getMemoryInfo(memory)
            return Device(kind(context.resources.configuration.smallestScreenWidthDp), megabytes(memory.totalMem))
        }
    }
}
