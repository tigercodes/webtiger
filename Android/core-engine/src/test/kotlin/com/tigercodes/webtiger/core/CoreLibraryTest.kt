// The JNI glue itself (Core/crates/wt-ffi/src/android.rs), below NativeCore: its constants
// against the C header, and what it does with bad messages, a listener that throws, and a stop
// asked from the wrong thread.
package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.CoreConfig
import com.tigercodes.webtiger.core.v1.CoreStatus
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.Envelope
import com.tigercodes.webtiger.core.v1.Ping
import com.tigercodes.webtiger.core.v1.Platform
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.TestMode
import java.io.File
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Keeps what the core delivered, and where. */
private class Collector(private val throwing: Boolean = false) : EnvelopeListener {
    val envelopes = CopyOnWriteArrayList<Envelope>()
    val threads = CopyOnWriteArrayList<Thread>()

    override fun onEnvelope(envelope: ByteArray) {
        envelopes.add(Envelope.ADAPTER.decode(envelope))
        threads.add(Thread.currentThread())
        if (throwing) throw IllegalStateException("a listener must not be able to hurt the core")
    }

    fun await(count: Int) {
        val deadline = System.nanoTime() + 10_000_000_000L
        while (envelopes.size < count) {
            check(System.nanoTime() < deadline) { "the core sent ${envelopes.size} of $count envelopes" }
            Thread.sleep(2)
        }
    }

    fun response(id: Long): Envelope {
        val deadline = System.nanoTime() + 10_000_000_000L
        while (true) {
            envelopes.firstOrNull { it.id == id && it.response != null }?.let { return it }
            check(System.nanoTime() < deadline) { "request $id was not answered" }
            Thread.sleep(2)
        }
    }
}

class CoreLibraryTest {
    private val config =
        CoreConfig(
            contract_major = NativeCore.CONTRACT_MAJOR,
            contract_minor = NativeCore.CONTRACT_MINOR,
            platform = Platform.PLATFORM_ANDROID,
            locale = "en-GB",
            test_mode = TestMode.TEST_MODE_FAKE,
            engines = listOf(EngineInfo(kind = EngineKind.ENGINE_KIND_CHROMIUM, version = "test")),
        ).encode()

    private fun ping(id: Long) = Envelope(id = id, request = Request(ping = Ping())).encode()

    /** The header's `#define NAME value` lines, values without their `u` suffix. */
    private fun defines(): Map<String, Long> {
        val header = File(checkNotNull(System.getProperty("webtiger.core.header")) { "the build names the header" })
        return header.readLines().mapNotNull { line ->
            val words = line.removePrefix("#define ").takeIf { it != line }?.trim()?.split(Regex("\\s+")) ?: return@mapNotNull null
            val value = words.getOrNull(1)?.trimEnd('u')?.toLongOrNull() ?: return@mapNotNull null
            words[0] to value
        }.toMap()
    }

    @Test
    fun theConstantsAreTheHeaders() {
        val defines = defines()
        val expected =
            mapOf(
                "WT_CONTRACT_MAJOR" to NativeCore.CONTRACT_MAJOR,
                "WT_CONTRACT_MINOR" to NativeCore.CONTRACT_MINOR,
                "WT_OK" to Status.OK,
                "WT_UNKNOWN_CORE" to Status.UNKNOWN_CORE,
                "WT_NULL_ARGUMENT" to Status.NULL_ARGUMENT,
                "WT_TOO_LARGE" to Status.TOO_LARGE,
                "WT_MALFORMED" to Status.MALFORMED,
                "WT_WRONG_THREAD" to Status.WRONG_THREAD,
                "WT_PANIC" to Status.PANIC,
                "WT_BAD_ARGUMENT" to Status.BAD_ARGUMENT,
                "WT_MAX_MESSAGE_LEN" to CoreLibrary.MAX_MESSAGE_LEN,
                "WT_VERDICT_ALLOW" to CoreLibrary.VERDICT_ALLOW,
                "WT_VERDICT_BLOCK" to CoreLibrary.VERDICT_BLOCK,
                "WT_NAV_MAIN_FRAME" to CoreLibrary.NAV_MAIN_FRAME,
                "WT_NAV_USER_GESTURE" to CoreLibrary.NAV_USER_GESTURE,
                "WT_NAV_ALLOW" to CoreLibrary.NAV_ALLOW,
                "WT_NAV_CANCEL" to CoreLibrary.NAV_CANCEL,
                "WT_NAV_EXTERNAL" to CoreLibrary.NAV_EXTERNAL,
            ) + ResourceType.entries.associate { "WT_RESOURCE_${it.name}" to it.code }
        for ((name, value) in expected) assertEquals(name, value.toLong(), defines[name])
        // A resource type or a status added to the header without its Kotlin name shows here.
        assertEquals(ResourceType.entries.size, defines.keys.count { it.startsWith("WT_RESOURCE_") })
        assertEquals((NativeCore.CONTRACT_MAJOR shl 16) or NativeCore.CONTRACT_MINOR, CoreLibrary.version())
    }

    @Test
    fun envelopesArriveInOrderOnTheCoresOneThread() {
        val collector = Collector()
        val core = CoreLibrary.start(config, collector)
        assertNotEquals(0L, core)
        try {
            for (id in 1L..200L) assertEquals(Status.OK, CoreLibrary.send(core, ping(id)))
            collector.await(201)
            assertEquals(CoreStatus.State.STATE_READY, collector.envelopes[0].event?.status?.state)
            assertEquals((1L..200L).toList(), collector.envelopes.drop(1).map { it.id })
            assertTrue(collector.envelopes.drop(1).all { it.response?.pong != null })
            assertEquals(1, collector.threads.distinct().size)
            assertNotEquals(Thread.currentThread(), collector.threads[0])
        } finally {
            assertEquals(Status.OK, CoreLibrary.stop(core))
        }
        // The handle names nothing any more.
        assertEquals(Status.UNKNOWN_CORE, CoreLibrary.stop(core))
        assertEquals(Status.UNKNOWN_CORE, CoreLibrary.send(core, ping(1)))
    }

    @Test
    fun whatIsNotAnEnvelopeIsRefusedAtOnce() {
        val collector = Collector()
        val core = CoreLibrary.start(config, collector)
        try {
            assertEquals(Status.MALFORMED, CoreLibrary.send(core, byteArrayOf(0x0A, 0x7F, 0x01)))
            // One byte longer than a message may be: refused without being read.
            assertEquals(Status.TOO_LARGE, CoreLibrary.send(core, ByteArray(CoreLibrary.MAX_MESSAGE_LEN + 1)))
            // The core is none the worse.
            assertEquals(Status.OK, CoreLibrary.send(core, ping(7)))
            assertTrue(collector.response(7).response?.pong != null)
        } finally {
            CoreLibrary.stop(core)
        }
        assertEquals(Status.UNKNOWN_CORE, CoreLibrary.send(0, ping(1)))
        assertEquals(Status.UNKNOWN_CORE, CoreLibrary.send(-1, ping(1)))
        assertEquals(Status.UNKNOWN_CORE, CoreLibrary.stop(-1))
    }

    @Test
    fun aRefusedStartReachesTheListenerBeforeStartReturns() {
        val collector = Collector()
        assertEquals(0L, CoreLibrary.start(byteArrayOf(0x0A, 0x7F, 0x01), collector))
        assertEquals(1, collector.envelopes.size)
        val status = collector.envelopes[0].event?.status
        assertEquals(CoreStatus.State.STATE_FATAL, status?.state)
        assertEquals("start.bad_config", status?.code)
        assertEquals(Thread.currentThread(), collector.threads[0])

        // A configuration too long to be a message starts nothing and is not read.
        val silent = Collector()
        assertEquals(0L, CoreLibrary.start(ByteArray(CoreLibrary.MAX_MESSAGE_LEN + 1), silent))
        assertTrue(silent.envelopes.isEmpty())
    }

    @Test
    fun aListenerThatThrowsDoesNotHurtTheCore() {
        val collector = Collector(throwing = true)
        val core = CoreLibrary.start(config, collector)
        assertNotEquals(0L, core)
        try {
            for (id in 1L..3L) assertEquals(Status.OK, CoreLibrary.send(core, ping(id)))
            for (id in 1L..3L) assertTrue(collector.response(id).response?.pong != null)
        } finally {
            assertEquals(Status.OK, CoreLibrary.stop(core))
        }
    }

    @Test
    fun aStopAskedFromInsideTheListenerIsRefused() {
        val asked = AtomicInteger(-1)
        var core = 0L
        val listener =
            EnvelopeListener { bytes ->
                if (Envelope.ADAPTER.decode(bytes).response != null) asked.set(CoreLibrary.stop(core))
            }
        core = CoreLibrary.start(config, listener)
        assertEquals(Status.OK, CoreLibrary.send(core, ping(1)))
        val deadline = System.nanoTime() + 10_000_000_000L
        while (asked.get() == -1 && System.nanoTime() < deadline) Thread.sleep(2)
        assertEquals(Status.WRONG_THREAD, asked.get())
        // The core still runs, and stops when asked from outside.
        assertEquals(Status.OK, CoreLibrary.stop(core))
    }

    @Test
    fun theFastPathFailsSafeWithoutACore() {
        val main = CoreLibrary.NAV_MAIN_FRAME
        assertEquals(CoreLibrary.NAV_CANCEL, CoreLibrary.navCheck(0, "https://example.com/", 1, main))
        assertEquals(CoreLibrary.NAV_CANCEL, CoreLibrary.navCheck(-5, "https://example.com/", 1, main))
        assertEquals(CoreLibrary.VERDICT_ALLOW, CoreLibrary.guardCheck(0, "https://example.com/a.js", "https://example.com/", 1, 3))

        val core = CoreLibrary.start(config, Collector())
        try {
            assertEquals(CoreLibrary.NAV_ALLOW, CoreLibrary.navCheck(core, "https://example.com/", 1, main))
            // Flags that cannot be flags are refused rather than read as "not the main frame".
            assertEquals(CoreLibrary.NAV_CANCEL, CoreLibrary.navCheck(core, "https://example.com/", 1, -1))
            assertEquals(CoreLibrary.VERDICT_ALLOW, CoreLibrary.guardCheck(core, "https://example.com/a.js", "https://example.com/", 1, 3))
        } finally {
            CoreLibrary.stop(core)
        }
        assertEquals(CoreLibrary.NAV_CANCEL, CoreLibrary.navCheck(core, "https://example.com/", 1, main))
    }

    @Test
    fun severalCoresRunSideBySideFromSeveralThreads() {
        val failures = CopyOnWriteArrayList<Throwable>()
        val threads =
            (1..4).map { number ->
                Thread {
                    try {
                        repeat(5) {
                            val collector = Collector()
                            val core = CoreLibrary.start(config, collector)
                            check(core != 0L) { "a core starts" }
                            for (id in 1L..20L) check(CoreLibrary.send(core, ping(id)) == Status.OK)
                            collector.response(20)
                            check(CoreLibrary.stop(core) == Status.OK)
                            check(collector.threads.distinct().size == 1) { "one thread per core" }
                        }
                    } catch (failure: Throwable) {
                        failures.add(failure)
                    }
                }.also { it.name = "shell-$number" }
            }
        threads.forEach { it.start() }
        threads.forEach { it.join(60_000) }
        assertEquals(emptyList<Throwable>(), failures.toList())
    }
}
