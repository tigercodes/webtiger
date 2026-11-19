// The system's own opener on a device (007): the activity it starts for an address, caught
// before it leaves the app, so the test starts no mail app or dialler on the device it runs on.
package com.tigercodes.webtiger

import android.app.Instrumentation
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.tigercodes.webtiger.ui.SystemOpener
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemOpenerTest {
    private val instrumentation = InstrumentationRegistry.getInstrumentation()
    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val monitors = ArrayList<Instrumentation.ActivityMonitor>()

    @After
    fun release() {
        for (monitor in monitors) instrumentation.removeMonitor(monitor)
    }

    /** Catches every activity started to view an address of `scheme`, and lets none through. */
    private fun catching(scheme: String): Instrumentation.ActivityMonitor {
        val views =
            IntentFilter(Intent.ACTION_VIEW).apply {
                addCategory(Intent.CATEGORY_BROWSABLE)
                addDataScheme(scheme)
            }
        return instrumentation.addMonitor(views, null, true).also(monitors::add)
    }

    @Test
    fun anAddressIsGivenToTheSystemToView() {
        val letters = catching("mailto")
        val numbers = catching("tel")

        assertTrue(SystemOpener(context).open("mailto:ada@example.com"))
        assertTrue(SystemOpener(context).open("tel:+15550100"))

        assertEquals("one activity was asked for, to view the letter's address", 1, letters.hits)
        assertEquals(1, numbers.hits)
    }

    @Test
    fun aSchemeNothingOnTheDeviceOpensIsSaidToBeSo() {
        // Asked of the system for real: no app is registered for this.
        assertFalse(SystemOpener(context).open("x-webtiger-test-no-such-scheme:1"))
    }
}
