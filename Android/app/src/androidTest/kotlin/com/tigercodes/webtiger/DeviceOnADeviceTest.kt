// What the app finds of the device it runs on (054), on a device: the kind and the memory it
// will say to the core, against what the device says of itself another way: its screen as the
// display measures it, and its memory as the kernel lists it.
package com.tigercodes.webtiger

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.core.v1.DeviceKind
import com.tigercodes.webtiger.ui.Device
import java.io.File
import kotlin.math.abs
import kotlin.math.min
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class DeviceOnADeviceTest {
    private val context = ApplicationProvider.getApplicationContext<Context>()

    @Test
    fun theKindIsTheOneTheScreenIsOf() {
        val device = Device.of(context)
        // The narrower side of the whole screen, in dp, as the display has it.
        val metrics = context.resources.displayMetrics
        val narrow = min(metrics.widthPixels, metrics.heightPixels) / metrics.density
        val expected = if (narrow >= Device.TABLET_FROM_DP) DeviceKind.DEVICE_KIND_TABLET else DeviceKind.DEVICE_KIND_PHONE
        assertEquals("a screen ${narrow}dp on its narrower side", expected, device.kind)
    }

    @Test
    fun theMemoryIsTheDevicesOwn() {
        val device = Device.of(context)
        // The kernel's own count, in kilobytes: "MemTotal:  2030224 kB".
        val listed = File("/proc/meminfo").readLines().first { it.startsWith("MemTotal:") }
        val megabytes = listed.filter(Char::isDigit).toLong() / 1024
        assertTrue("the kernel lists some: $listed", megabytes > 256)
        assertTrue(
            "the app says ${device.megabytes} megabytes where the kernel lists $megabytes",
            abs(device.megabytes - megabytes) <= megabytes / 20,
        )
    }
}
