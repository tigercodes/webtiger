// What the system says of its memory reaches the core as the right word (055): before Android
// 14 the system tells a running app, and from Android 14 the app must ask. No test makes a
// system short of memory: the system's callbacks are called as the system calls them, and its
// answer to the question is the test's. What the core does with the word is the core's
// (memory_short.rs); that it reaches the core is ThroughTheCoreTest's; what a device's own
// system says is the device suite's (MemoryOnADeviceTest).
package com.tigercodes.webtiger

import android.content.ComponentCallbacks2
import com.tigercodes.webtiger.core.v1.MemoryLevel
import com.tigercodes.webtiger.ui.MemoryWatch
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class MemoryWatchTest {
    private val scope = TestScope()
    private val said = ArrayList<MemoryLevel>()
    private var low = false
    private var asked = 0

    private fun watch(asks: Boolean) =
        MemoryWatch(
            scope = scope,
            asks = asks,
            isLow = {
                asked += 1
                low
            },
            say = said::add,
        )

    private fun pass(seconds: Int) {
        scope.advanceTimeBy(seconds.seconds)
        scope.runCurrent()
    }

    @Suppress("DEPRECATION")
    @Test
    fun runningLowIsShortAndRunningCriticalVeryShortAndNothingElseIsAWord() {
        assertEquals(MemoryLevel.MEMORY_LEVEL_SHORT, MemoryWatch.level(ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW))
        assertEquals(MemoryLevel.MEMORY_LEVEL_VERY_SHORT, MemoryWatch.level(ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL))
        // That the system is a little short, that the window is hidden, or that the process is
        // one of those kept in the background, says nothing the core is told.
        val others =
            listOf(
                ComponentCallbacks2.TRIM_MEMORY_RUNNING_MODERATE,
                ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN,
                ComponentCallbacks2.TRIM_MEMORY_BACKGROUND,
                ComponentCallbacks2.TRIM_MEMORY_MODERATE,
                ComponentCallbacks2.TRIM_MEMORY_COMPLETE,
                0,
                -1,
                11,
            )
        for (other in others) assertNull("level $other", MemoryWatch.level(other))
    }

    @Suppress("DEPRECATION")
    @Test
    fun whatTheSystemTellsARunningAppIsPassedOnEachTime() {
        val watch = watch(asks = false)
        watch.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW)
        watch.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN)
        watch.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL)
        watch.onTrimMemory(ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW)
        assertEquals(
            listOf(MemoryLevel.MEMORY_LEVEL_SHORT, MemoryLevel.MEMORY_LEVEL_VERY_SHORT, MemoryLevel.MEMORY_LEVEL_SHORT),
            said,
        )
        // "Low memory", before Android 14: every process behind has been ended. The last word.
        watch.onLowMemory()
        assertEquals(MemoryLevel.MEMORY_LEVEL_VERY_SHORT, said.last())
        assertEquals(4, said.size)
    }

    @Test
    fun whereTheSystemTellsNothingItIsAskedEveryMinuteWhileTheAppIsInFront() {
        val watch = watch(asks = true)
        // Not in front yet: nothing is asked, however long.
        pass(600)
        assertEquals(0, asked)

        // In front. Not asked for a minute; then once a minute. The answer is no: nothing said.
        watch.inFront(true)
        pass(59)
        assertEquals(0, asked)
        pass(1)
        assertEquals(1, asked)
        pass(120)
        assertEquals(3, asked)
        assertEquals(emptyList<MemoryLevel>(), said)

        // The answer turns to yes: short, each minute for as long as it is yes. A system that
        // is short still says so again (055), and here asking is how it says it.
        low = true
        pass(60)
        assertEquals(listOf(MemoryLevel.MEMORY_LEVEL_SHORT), said)
        pass(120)
        assertEquals(List(3) { MemoryLevel.MEMORY_LEVEL_SHORT }, said)
        low = false
        pass(180)
        assertEquals(3, said.size)

        // Told twice that it is in front, it asks once a minute still.
        val before = asked
        watch.inFront(true)
        pass(60)
        assertEquals(before + 1, asked)

        // Out of sight: nothing is asked, and the app is not kept busy. In front again: asked
        // a minute on.
        watch.inFront(false)
        low = true
        pass(600)
        assertEquals(before + 1, asked)
        assertEquals(3, said.size)
        watch.inFront(true)
        pass(60)
        assertEquals(before + 2, asked)
        assertEquals(4, said.size)
        watch.inFront(false)
    }

    @Test
    fun whereTheSystemTellsARunningAppItIsNotAsked() {
        val watch = watch(asks = false)
        low = true
        watch.inFront(true)
        pass(600)
        assertEquals(0, asked)
        assertEquals(emptyList<MemoryLevel>(), said)
        watch.inFront(false)
    }

    @Test
    fun theSystemIsAskedOnceAMinuteAndAndroidFourteenIsTheFirstThatMustBe() {
        assertEquals(1.minutes, MemoryWatch.ASK_EVERY)
        assertEquals(34, MemoryWatch.TELLS_NOTHING_FROM)
    }
}
