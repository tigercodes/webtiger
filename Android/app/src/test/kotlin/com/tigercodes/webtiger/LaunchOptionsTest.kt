package com.tigercodes.webtiger

import android.content.Intent
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.ui.LaunchOptions
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/** Fake mode on Android: an intent's extras, in a debuggable build (003 §7). */
@RunWith(RobolectricTestRunner::class)
class LaunchOptionsTest {
    private fun fake(start: String? = null) =
        Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_FAKE, true).apply { if (start != null) putExtra(MainActivity.EXTRA_FAKE_START, start) }

    @Test
    fun anOrdinaryLaunchIsLive() {
        assertEquals(LaunchOptions(), MainActivity.launchOptions(Intent(Intent.ACTION_MAIN), debuggable = true))
        assertEquals(LaunchOptions(), MainActivity.launchOptions(null, debuggable = true))
        // The start address alone asks for nothing.
        val startOnly = Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_FAKE_START, "http://127.0.0.1:8080/")
        assertEquals(LaunchOptions(), MainActivity.launchOptions(startOnly, debuggable = true))
    }

    @Test
    fun theExtrasAskForFakeModeAndItsStartAddress() {
        assertEquals(LaunchOptions(fake = true), MainActivity.launchOptions(fake(), debuggable = true))
        assertEquals(
            LaunchOptions(fake = true, fakeStart = "http://127.0.0.1:8080/start"),
            MainActivity.launchOptions(fake("http://127.0.0.1:8080/start"), debuggable = true),
        )
    }

    /**
     * Fake mode's numbers: the clock's step (049), the sleep time (053), the limit of live tabs
     * (054), how often the core is told the system is short of memory (055).
     */
    @Test
    fun theExtrasGiveFakeModeItsNumbers() {
        val numbers =
            fake()
                .putExtra(MainActivity.EXTRA_FAKE_CLOCK_STEP, 86_400)
                .putExtra(MainActivity.EXTRA_FAKE_SLEEP_AFTER, 5)
                .putExtra(MainActivity.EXTRA_FAKE_LIVE_TABS, 2)
                .putExtra(MainActivity.EXTRA_FAKE_MEMORY_SHORT_EVERY, 3)
        assertEquals(
            LaunchOptions(fake = true, clockStep = 86_400, sleepAfter = 5, liveTabs = 2, memoryShortEvery = 3),
            MainActivity.launchOptions(numbers, debuggable = true),
        )
        // And the standard session (060): asked for, it is; said to be false, or not said, it is not.
        assertEquals(
            LaunchOptions(fake = true, standardSession = true),
            MainActivity.launchOptions(fake().putExtra(MainActivity.EXTRA_FAKE_STANDARD, true), debuggable = true),
        )
        assertEquals(
            LaunchOptions(fake = true),
            MainActivity.launchOptions(fake().putExtra(MainActivity.EXTRA_FAKE_STANDARD, false), debuggable = true),
        )
        assertEquals(
            "without fake mode there is no such thing",
            LaunchOptions(),
            MainActivity.launchOptions(Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_FAKE_STANDARD, true), debuggable = true),
        )
        // And a list for the core to open by itself (063): in its order, each address as it was
        // written, with the seconds between.
        val list =
            fake()
                .putExtra(MainActivity.EXTRA_FAKE_OPEN, "https://a.example.com/  https://b.example.com/one?two=3\nhttp://c.example.com/\n")
                .putExtra(MainActivity.EXTRA_FAKE_OPEN_EVERY, 20)
        assertEquals(
            LaunchOptions(
                fake = true,
                open = listOf("https://a.example.com/", "https://b.example.com/one?two=3", "http://c.example.com/"),
                openEvery = 20,
            ),
            MainActivity.launchOptions(list, debuggable = true),
        )
        assertEquals(
            "nothing but spaces is no list, and seconds below nothing are none",
            LaunchOptions(fake = true),
            MainActivity.launchOptions(
                fake().putExtra(MainActivity.EXTRA_FAKE_OPEN, " \n ").putExtra(MainActivity.EXTRA_FAKE_OPEN_EVERY, -5),
                debuggable = true,
            ),
        )
        assertEquals(LaunchOptions(), MainActivity.launchOptions(list, debuggable = false))
        // And the device's own limit of live tabs, for a run of the app as it is (063).
        assertEquals(
            LaunchOptions(fake = true, deviceLimits = true),
            MainActivity.launchOptions(fake().putExtra(MainActivity.EXTRA_FAKE_DEVICE_LIMITS, true), debuggable = true),
        )
        // A number below nothing is nothing; and without fake mode none of them is anything.
        val below =
            fake()
                .putExtra(MainActivity.EXTRA_FAKE_LIVE_TABS, -3)
                .putExtra(MainActivity.EXTRA_FAKE_SLEEP_AFTER, -1)
                .putExtra(MainActivity.EXTRA_FAKE_MEMORY_SHORT_EVERY, -2)
        assertEquals(LaunchOptions(fake = true), MainActivity.launchOptions(below, debuggable = true))
        val alone = Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_FAKE_LIVE_TABS, 2)
        assertEquals(LaunchOptions(), MainActivity.launchOptions(alone, debuggable = true))
        assertEquals(LaunchOptions(), MainActivity.launchOptions(numbers, debuggable = false))
    }

    @Test
    fun aBuildThatIsNotDebuggableIgnoresThem() {
        assertEquals(LaunchOptions(), MainActivity.launchOptions(fake("http://127.0.0.1:8080/start"), debuggable = false))
        val catalogue = Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_CATALOGUE, true)
        assertEquals(LaunchOptions(), MainActivity.launchOptions(catalogue, debuggable = false))
    }

    @Test
    fun anExtraAsksForTheDesignCatalogue() {
        val catalogue = Intent(Intent.ACTION_MAIN).putExtra(MainActivity.EXTRA_CATALOGUE, true)
        assertEquals(LaunchOptions(catalogue = true), MainActivity.launchOptions(catalogue, debuggable = true))
    }

    @Test
    fun aFailedStartIsShownAsACode() {
        assertEquals("store.open", WebTigerApp.startCode(CoreError.StartRefused("store.open")))
        assertEquals("contract.mismatch", WebTigerApp.startCode(CoreError.ContractMismatch(library = 2 shl 16)))
        // The core's library is not in the app, or would not load: its words stay out of the window.
        assertEquals("core.unavailable", WebTigerApp.startCode(UnsatisfiedLinkError("dlopen failed: /data/app/…/libwebtiger_core.so")))
    }
}
