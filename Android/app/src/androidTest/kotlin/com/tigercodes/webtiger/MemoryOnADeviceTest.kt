// What a device's own system says of its memory reaches the app's watch (055), on a device.
// Nothing is made short of memory: the application is told what the system would tell it, by
// the application's own way of being told, and the system is asked its own question for real.
// (What the core then does is the core's, and ThroughTheCoreTest's on the JVM.)
package com.tigercodes.webtiger

import android.app.ActivityManager
import android.app.Application
import android.content.ComponentCallbacks2
import android.content.Context
import android.os.Build
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.tigercodes.webtiger.core.v1.MemoryLevel
import com.tigercodes.webtiger.ui.MemoryWatch
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class MemoryOnADeviceTest {
    private val application = ApplicationProvider.getApplicationContext<Context>() as Application
    private val instrumentation = InstrumentationRegistry.getInstrumentation()

    /** A watch that is registered with the application hears what the application is told, as the right word. */
    @Suppress("DEPRECATION")
    @Test
    fun whatTheApplicationIsToldOfMemoryReachesAWatchRegisteredWithIt() {
        val said = CopyOnWriteArrayList<MemoryLevel>()
        val scope = MainScope()
        val watch = MemoryWatch(scope = scope, asks = false, isLow = { false }, say = said::add)
        application.registerComponentCallbacks(watch)
        try {
            // On the main thread, where the system tells an application.
            instrumentation.runOnMainSync {
                application.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW)
                application.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_MODERATE)
                application.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL)
            }
            assertEquals(listOf(MemoryLevel.MEMORY_LEVEL_SHORT, MemoryLevel.MEMORY_LEVEL_VERY_SHORT), said.toList())
        } finally {
            application.unregisterComponentCallbacks(watch)
            scope.cancel()
        }
        // No longer registered, it hears nothing.
        instrumentation.runOnMainSync { application.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW) }
        assertEquals(2, said.size)
    }

    /**
     * The device's own system is asked its question, again and again while the app is in front,
     * on the main thread where the app asks it; and what the watch says is what the system
     * answered. (An emulator at rest answers no; a device that is short answers yes, and then
     * short is said, and nothing else.)
     */
    @Test
    fun theSystemIsAskedItsOwnQuestionWhileTheAppIsInFront() {
        val direct = ActivityManager.MemoryInfo().also { application.getSystemService(ActivityManager::class.java).getMemoryInfo(it) }
        assertTrue("the device has a threshold below which it is low: ${direct.threshold}", direct.threshold > 0)
        val asked = AtomicInteger()
        val yes = AtomicInteger()
        val said = CopyOnWriteArrayList<MemoryLevel>()
        val scope = MainScope()
        val watch =
            MemoryWatch(
                scope = scope,
                asks = Build.VERSION.SDK_INT >= MemoryWatch.TELLS_NOTHING_FROM,
                isLow = {
                    asked.incrementAndGet()
                    MemoryWatch.isLow(application).also { if (it) yes.incrementAndGet() }
                },
                every = 50.milliseconds,
                say = said::add,
            )
        try {
            instrumentation.runOnMainSync { watch.inFront(true) }
            Thread.sleep(1_500)
            instrumentation.runOnMainSync { watch.inFront(false) }
        } finally {
            scope.cancel()
        }
        val times = asked.get()
        if (Build.VERSION.SDK_INT >= MemoryWatch.TELLS_NOTHING_FROM) {
            assertTrue("asked every so often while in front: $times times", times in 5..40)
        } else {
            assertEquals("a system that tells a running app is not asked", 0, times)
        }
        assertEquals("short is said each time the answer is yes, and only then", yes.get(), said.size)
        assertTrue(said.all { it == MemoryLevel.MEMORY_LEVEL_SHORT })
        // Out of sight, it asks no more.
        Thread.sleep(300)
        assertEquals(times, asked.get())
    }
}
