// What the app says it runs on (054): a phone or a tablet by the narrower side of its screen,
// and its memory in megabytes. What the core makes of it is the core's (live_tabs.rs); that
// it reaches the core is ThroughTheCoreTest's; what a device says of itself is the device
// suite's (DeviceOnADeviceTest).
package com.tigercodes.webtiger

import com.tigercodes.webtiger.core.v1.DeviceKind
import com.tigercodes.webtiger.ui.Device
import org.junit.Assert.assertEquals
import org.junit.Test

class DeviceTest {
    @Test
    fun aScreenSixHundredDpOnItsNarrowerSideIsATabletsAndANarrowerOneAPhones() {
        for (narrow in listOf(320, 411, 599)) assertEquals("$narrow dp", DeviceKind.DEVICE_KIND_PHONE, Device.kind(narrow))
        for (narrow in listOf(600, 601, 800, 1_280)) assertEquals("$narrow dp", DeviceKind.DEVICE_KIND_TABLET, Device.kind(narrow))
        // A screen of no known size (the system says 0) is taken for a phone's: the fewer live tabs.
        assertEquals(DeviceKind.DEVICE_KIND_PHONE, Device.kind(0))
    }

    @Test
    fun memoryIsSaidInWholeMegabytes() {
        assertEquals(8_192, Device.megabytes(8L * 1024 * 1024 * 1024))
        assertEquals(5_859, Device.megabytes(6_144_000_000L))
        assertEquals(0, Device.megabytes(1024 * 1024 - 1))
        assertEquals("a number that is no size is none", 0, Device.megabytes(-1))
        assertEquals(0, Device.megabytes(-5_000_000_000L))
        assertEquals("and never more than the contract's field holds", Int.MAX_VALUE, Device.megabytes(Long.MAX_VALUE))
    }
}
