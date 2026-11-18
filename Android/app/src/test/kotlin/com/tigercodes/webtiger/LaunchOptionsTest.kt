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
