// The adapter's own fetcher (027): cookies stay with their site. The walk from a download's
// address to its file, with scripted sites: who is asked, with whose cookies, what is kept of
// what a site sets, where the following stops, and what becomes of a file that is not whole.
// And the fetcher itself, over real connections to a server in this process that has two names:
// two sites.
package com.tigercodes.webtiger.webview

import java.io.ByteArrayInputStream
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.URI
import java.util.concurrent.CopyOnWriteArrayList
import kotlin.concurrent.thread
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

private const val HERE = "https://example.com/files/report"
private const val THERE = "https://cdn.example.net/2026/report.pdf"

/** A sink in memory: what was written, and what became of it. */
private class Kept(private val name: String = "report.pdf", private val full: Int = Int.MAX_VALUE, private val unfinishable: Boolean = false) : Sink {
    val bytes = ArrayList<Byte>()
    var whole = false
    var discarded = false

    override fun write(bytes: ByteArray, count: Int) {
        if (this.bytes.size + count > full) throw IOException("no room")
        for (index in 0 until count) this.bytes.add(bytes[index])
    }

    override fun finish(): String {
        if (unfinishable) throw IOException("cannot be closed")
        whole = true
        return name
    }

    override fun discard() {
        discarded = true
    }
}

class AppFetcherTest {
    @get:Rule val folder = TemporaryFolder()

    /** Who was asked, in order: the address, and the cookies it was asked with. */
    private val asked = ArrayList<Pair<String, String?>>()

    /** What each site knows the person by: the host, and its cookies. */
    private val cookies = HashMap(mapOf("example.com" to "who=ada", "cdn.example.net" to "seen=before"))

    /** What sites set on the way: the address that set it, and the cookie. */
    private val set = ArrayList<Pair<String, String>>()

    private val sink = Kept()
    private var opened = 0

    private fun host(address: String): String = URI(address).host.orEmpty()

    private fun walk(url: String, answers: Map<String, () -> Reply?>, into: Sink? = sink, stopped: () -> Boolean = { false }, progress: (Long, Long) -> Unit = { _, _ -> }): Fetched =
        fetch(
            url,
            Fetching(
                cookiesFor = { address -> cookies[host(address)] },
                keep = { address, cookie ->
                    set.add(address to cookie)
                    // As a cookie store would: the site's own cookie, for the site that set it.
                    cookies[host(address)] = cookie.substringBefore(';')
                },
                ask = { address, with ->
                    asked.add(address to with)
                    answers[address]?.invoke()
                },
                sink = {
                    opened += 1
                    into
                },
                stopped = stopped,
                progress = progress,
            ),
        )

    private fun file(text: String, status: Int = 200, length: Long = text.length.toLong(), cookies: List<String> = emptyList()): () -> Reply = { Reply(status, body = ByteArrayInputStream(text.toByteArray()), length = length, cookies = cookies) }

    private fun sendsOn(to: String?, status: Int = 302, cookies: List<String> = emptyList()): () -> Reply = { Reply(status, location = to, cookies = cookies) }

    private fun saved(): String = String(sink.bytes.toByteArray())

    @Test
    fun aFileWhereItsAddressSaysIsFetchedAsItsSiteKnowsThePerson() {
        val seen = ArrayList<Pair<Long, Long>>()

        val ended = walk(HERE, mapOf(HERE to file("the report")), progress = { received, expected -> seen.add(received to expected) })

        assertEquals(Fetched.Whole("report.pdf"), ended)
        assertEquals("the report", saved())
        assertTrue(sink.whole)
        assertFalse(sink.discarded)
        assertEquals(listOf(HERE to "who=ada"), asked)
        assertEquals("how far it got was said, to the end", listOf(0L to 10L, 10L to 10L), seen)
    }

    @Test
    fun aFileThatIsSomewhereElseIsFetchedThereWithThatSitesCookiesAndNoOthers() {
        val ended = walk(HERE, mapOf(HERE to sendsOn(THERE), THERE to file("the report")))

        assertEquals(Fetched.Whole("report.pdf"), ended)
        assertEquals("the report", saved())
        // What the first site knows the person by was not sent to the second.
        assertEquals(listOf(HERE to "who=ada", THERE to "seen=before"), asked)
    }

    @Test
    fun aSiteThePersonIsNobodyAtIsAskedWithNoCookies() {
        val nobody = "https://files.example.org/report.pdf"

        walk(HERE, mapOf(HERE to sendsOn(nobody), nobody to file("the report")))

        assertEquals(listOf(HERE to "who=ada", nobody to null), asked)
    }

    @Test
    fun everyKindOfSendingOnIsFollowedByHandAndAPathIsWithinItsOwnSite() {
        for (status in listOf(301, 302, 303, 307, 308)) {
            asked.clear()
            val next = "https://example.com/files/2026/report.pdf"

            val ended = walk(HERE, mapOf(HERE to sendsOn("2026/report.pdf", status), next to file("the report")))

            assertEquals("$status", Fetched.Whole("report.pdf"), ended)
            // The same site: asked as the site knows the person, both times.
            assertEquals("$status", listOf(HERE to "who=ada", next to "who=ada"), asked)
        }
    }

    @Test
    fun aCookieASiteSetsOnTheWayIsKeptForThatSiteBeforeTheNextAddressIsAsked() {
        val next = "https://example.com/files/2026/report.pdf"

        walk(
            HERE,
            mapOf(
                HERE to sendsOn(next, cookies = listOf("who=grace; Path=/", "seen=today")),
                next to sendsOn(THERE),
                THERE to file("the report", cookies = listOf("last=now")),
            ),
        )

        // Each as its site wrote it, for the address that set it.
        assertEquals(listOf(HERE to "who=grace; Path=/", HERE to "seen=today", THERE to "last=now"), set)
        // The site that set it sees it at its next address; the other site sees its own, and
        // nothing of the first's.
        assertEquals(listOf(HERE to "who=ada", next to "seen=today", THERE to "seen=before"), asked)
    }

    @Test
    fun followingStopsAfterTenTimesAndAtAnAddressThatIsNotTheWebs() {
        // Round and round: asked eleven times, the first and ten more, and no further.
        val round = walk(HERE, mapOf(HERE to sendsOn(HERE)))
        assertEquals(Fetched.Failed("download.network"), round)
        assertEquals(11, asked.size)
        assertEquals("nothing was saved, and nothing made to save in", 0, opened)

        for (elsewhere in listOf("file:///data/secret", "content://media/1", "javascript:alert(1)", "intent://x#Intent;end", "ftp://example.com/f", " ")) {
            asked.clear()
            val ended = walk(HERE, mapOf(HERE to sendsOn(elsewhere)))
            assertEquals(elsewhere, Fetched.Failed("download.network"), ended)
            assertEquals("nobody else was asked", listOf(HERE to "who=ada"), asked)
        }
        // Sent on to nowhere at all.
        asked.clear()
        assertEquals(Fetched.Failed("download.network"), walk(HERE, mapOf(HERE to sendsOn(null))))
        assertEquals(0, opened)
    }

    @Test
    fun whatIsNoFileIsAFailureAndNothingIsSaved() {
        // Nobody answered; a site that says no; one that says it is not there; one that says
        // all is well and sends nothing.
        val answers: List<Map<String, () -> Reply?>> =
            listOf(
                emptyMap(),
                mapOf(HERE to { null }),
                mapOf(HERE to { Reply(403) }),
                mapOf(HERE to { Reply(404, body = ByteArrayInputStream("not here".toByteArray())) }),
                mapOf(HERE to { Reply(500) }),
                mapOf(HERE to { Reply(200) }),
                // An answer this fetcher follows nobody for: it is no file.
                mapOf(HERE to { Reply(300, location = THERE) }),
                mapOf(HERE to { Reply(304) }),
            )
        for (answer in answers) {
            assertEquals(Fetched.Failed("download.network"), walk(HERE, answer))
        }
        assertEquals(0, opened)
        assertEquals("", saved())
    }

    @Test
    fun aFileThatIsNotWholeIsNotLeftBehind() {
        // Fewer bytes than the site said.
        assertEquals(Fetched.Failed("download.network"), walk(HERE, mapOf(HERE to file("half", length = 4096))))
        assertTrue(sink.discarded)
        assertFalse(sink.whole)

        // The connection ends while the file is coming.
        val broken = Kept()
        val breaks =
            object : InputStream() {
                private var given = false

                override fun read(): Int = throw IOException("gone")

                override fun read(bytes: ByteArray): Int {
                    if (given) throw IOException("gone")
                    given = true
                    bytes[0] = 'h'.code.toByte()
                    return 1
                }
            }
        assertEquals(Fetched.Failed("download.network"), walk(HERE, mapOf(HERE to { Reply(200, body = breaks) }), into = broken))
        assertTrue(broken.discarded)

        // No room for it; a file that cannot be finished; and nowhere to save at all.
        val small = Kept(full = 3)
        assertEquals(Fetched.Failed("download.disk"), walk(HERE, mapOf(HERE to file("too much")), into = small))
        assertTrue(small.discarded)
        val stuck = Kept(unfinishable = true)
        assertEquals(Fetched.Failed("download.disk"), walk(HERE, mapOf(HERE to file("all of it")), into = stuck))
        assertTrue(stuck.discarded)
        assertEquals(Fetched.Failed("download.disk"), walk(HERE, mapOf(HERE to file("all of it")), into = null))
    }

    @Test
    fun aFetchThatIsStoppedAsksNobodyElseAndLeavesNothing() {
        // Stopped before it began.
        assertEquals(Fetched.Stopped, walk(HERE, mapOf(HERE to file("the report")), stopped = { true }))
        assertEquals(emptyList<Pair<String, String?>>(), asked)

        // Stopped on the way: the next site is not asked.
        var stop = false
        val onTheWay = walk(HERE, mapOf(HERE to { stop = true; Reply(302, location = THERE) }, THERE to file("the report")), stopped = { stop })
        assertEquals(Fetched.Stopped, onTheWay)
        assertEquals(listOf(HERE to "who=ada"), asked)

        // Stopped while the file is coming: what had come is removed.
        var coming = false
        val partWay = walk(HERE, mapOf(HERE to file("the report")), stopped = { coming }, progress = { _, _ -> coming = true })
        assertEquals(Fetched.Stopped, partWay)
        assertTrue(sink.discarded)
        assertFalse(sink.whole)
    }

    // The files a download is saved in.

    @Test
    fun aFileIsSavedUnderItsNameAndANameThatIsTakenGainsANumber() {
        val downloads = folder.newFolder("downloads")
        val names =
            (1..3).map {
                val file = checkNotNull(FileSink.open(downloads, "notes.txt"))
                file.write("n$it".toByteArray(), 2)
                file.finish()
            }

        assertEquals(listOf("notes.txt", "notes-1.txt", "notes-2.txt"), names)
        assertEquals(listOf("n1", "n2", "n3"), names.map { File(downloads, it).readText() })
        // A name with no ending, and one that begins with a dot.
        File(downloads, "README").writeText("taken")
        assertEquals("README-1", checkNotNull(FileSink.open(downloads, "README")).finish())
        // A file that is not wanted is gone, and its name is free again.
        val dropped = checkNotNull(FileSink.open(downloads, "dropped.bin"))
        dropped.write(byteArrayOf(1, 2, 3), 3)
        dropped.discard()
        assertFalse(File(downloads, "dropped.bin").exists())
        // A folder that is not there yet is made; a place that is no folder gives no file.
        assertEquals("first.txt", checkNotNull(FileSink.open(File(downloads, "deeper/still"), "first.txt")).finish())
        assertNull(FileSink.open(File(downloads, "README"), "inside.txt"))
    }

    // The fetcher itself, over real connections: a server in this process, which 127.0.0.1 and
    // localhost both name. They are two sites, with a cookie each.

    private val site = TwoNames()

    @After
    fun close() = site.close()

    /** A login set's cookies, by the host they are for: what a cookie store would say. */
    private class HostJar(private val byHost: MutableMap<String, String>) : Jar {
        val kept = CopyOnWriteArrayList<Pair<String, String>>()

        override fun cookiesFor(address: String): String? = byHost[URI(address).host]

        override fun keep(address: String, cookie: String) {
            kept.add(URI(address).host to cookie)
            byHost[URI(address).host] = cookie.substringBefore(';')
        }
    }

    private fun await(fetcher: AppFetcher, handle: Long): FetchStatus {
        repeat(500) {
            val status = fetcher.status(handle)
            if (status !is FetchStatus.Coming) return checkNotNull(status) { "the fetcher forgot the fetch" }
            Thread.sleep(10)
        }
        error("the fetch never ended")
    }

    private fun fetcher(jar: Jar?, into: File): AppFetcher = AppFetcher(jars = { jar }, sinks = { name -> FileSink.open(into, name) })

    @Test
    fun overRealConnectionsOneSitesCookiesAreNotSentToTheSiteItSendsOnTo() {
        val downloads = folder.newFolder("fetched")
        val jar = HostJar(hashMapOf("127.0.0.1" to "who=ada"))
        val fetcher = fetcher(jar, downloads)

        // A file of this site, that says who asked for it.
        val mine = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/whose-file"), "TestAgent/1", "", "", 0), "set", "mine.txt")
        assertEquals(FetchStatus.Finished("mine.txt"), await(fetcher, checkNotNull(mine)))
        assertEquals("who=ada agent=TestAgent/1", File(downloads, "mine.txt").readText())

        // The same file, at an address of this site that sends on to the other site: the
        // person is nobody there, and what this site knows them by was not sent.
        val elsewhere = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/to-the-other"), "TestAgent/1", "", "", 0), "set", "elsewhere.txt")
        assertEquals(FetchStatus.Finished("elsewhere.txt"), await(fetcher, checkNotNull(elsewhere)))
        assertEquals("who=none agent=TestAgent/1", File(downloads, "elsewhere.txt").readText())

        // Signed in at the other site too, as somebody else: it is asked as it knows them.
        jar.keep("http://localhost/", "who=grace")
        val theirs = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/to-the-other"), "TestAgent/1", "", "", 0), "set", "theirs.txt")
        assertEquals(FetchStatus.Finished("theirs.txt"), await(fetcher, checkNotNull(theirs)))
        assertEquals("who=grace agent=TestAgent/1", File(downloads, "theirs.txt").readText())

        // Every request the server had: nothing but the cookie of the name it was asked by, no
        // sign-in, and no address it was sent from.
        val leaked = site.requests.filter { it.host.startsWith("localhost") && it.cookie.contains("ada") }
        assertEquals("the first site's cookie never reached the second", emptyList<TwoNames.Request>(), leaked)
        assertTrue(site.requests.all { it.authorization == null && it.referer == null })
    }

    @Test
    fun overRealConnectionsACookieSetOnTheWayIsKeptForItsSiteAndSeenByIt() {
        val downloads = folder.newFolder("signed")
        val jar = HostJar(hashMapOf())
        val fetcher = fetcher(jar, downloads)

        val handle = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/signs-in-and-sends?v=lin"), "", "", "", 0), "set", "who.txt")

        assertEquals(FetchStatus.Finished("who.txt"), await(fetcher, checkNotNull(handle)))
        assertEquals("who=lin agent=", File(downloads, "who.txt").readText().substringBefore("Java").substringBefore("okhttp"))
        assertEquals(listOf("127.0.0.1" to "who=lin; Path=/"), jar.kept.toList())
    }

    @Test
    fun overRealConnectionsAFailureIsACodeAndLeavesNoFile() {
        val downloads = folder.newFolder("failed")
        val fetcher = fetcher(null, downloads)

        // A page that is not there; an address of no web; a server that is not there.
        val missing = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/missing"), "", "", "", 0), "set", "missing.txt")
        assertEquals(FetchStatus.Failed("download.network"), await(fetcher, checkNotNull(missing)))
        assertNull("only an address of the web is fetched", fetcher.start(DownloadOffer("file:///etc/hosts", "", "", "", 0), "set", "hosts.txt"))
        val free = ServerSocket(0).use { it.localPort }
        val nobody = fetcher.start(DownloadOffer("http://127.0.0.1:$free/file", "", "", "", 0), "set", "nobody.txt")
        assertEquals(FetchStatus.Failed("download.network"), await(fetcher, checkNotNull(nobody)))
        assertEquals(emptyList<String>(), downloads.list().orEmpty().toList())

        // What has ended is told once; a fetch nobody started is not known.
        assertNull(fetcher.status(checkNotNull(missing)))
        assertNull(fetcher.status(99))
    }

    @Test
    fun aNameIsHeldToANameWhateverItSays() {
        val downloads = folder.newFolder("named")
        val fetcher = fetcher(null, downloads)

        val handle = fetcher.start(DownloadOffer(site.url("127.0.0.1", "/whose-file"), "", "", "", 0), "set", "../../escaped.txt")

        assertEquals(FetchStatus.Finished("escaped.txt"), await(fetcher, checkNotNull(handle)))
        assertEquals(listOf("escaped.txt"), downloads.list().orEmpty().toList())
    }
}

/**
 * A server on the loopback address, which both of its names reach: "127.0.0.1" and "localhost"
 * are two sites to a browser. Its pages say who asked, and one sends on to the other name.
 */
private class TwoNames : AutoCloseable {
    class Request(val host: String, val path: String, val cookie: String, val authorization: String?, val referer: String?)

    private val server = ServerSocket(0, 50, InetAddress.getByName("127.0.0.1"))
    val requests = CopyOnWriteArrayList<Request>()

    init {
        thread(isDaemon = true, name = "two-names") {
            while (!server.isClosed) {
                val socket =
                    try {
                        server.accept()
                    } catch (_: IOException) {
                        return@thread
                    }
                thread(isDaemon = true) {
                    socket.use {
                        val lines = it.getInputStream().bufferedReader().let { reader -> generateSequence { reader.readLine() }.takeWhile { line -> line.isNotEmpty() }.toList() }
                        if (lines.isEmpty()) return@use
                        val path = lines.first().split(' ').getOrElse(1) { "/" }

                        fun header(name: String): String? = lines.drop(1).firstOrNull { line -> line.startsWith("$name:", ignoreCase = true) }?.substringAfter(':')?.trim()
                        val request = Request(header("Host").orEmpty(), path, header("Cookie").orEmpty(), header("Authorization"), header("Referer"))
                        requests.add(request)
                        val who = request.cookie.split(';').map { part -> part.trim() }.firstOrNull { part -> part.startsWith("who=") } ?: "who=none"
                        val answer =
                            when {
                                path == "/whose-file" -> ok("$who agent=${header("User-Agent").orEmpty()}")
                                path == "/to-the-other" -> "HTTP/1.1 302 Found\r\nLocation: http://localhost:${server.localPort}/whose-file\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                                path.startsWith("/signs-in-and-sends") ->
                                    "HTTP/1.1 302 Found\r\nSet-Cookie: who=${path.substringAfter("v=", "someone")}; Path=/\r\nLocation: /whose-file\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                                else -> "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            }
                        it.getOutputStream().apply {
                            write(answer.toByteArray())
                            flush()
                        }
                    }
                }
            }
        }
    }

    private fun ok(body: String): String = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ${body.toByteArray().size}\r\nConnection: close\r\n\r\n$body"

    fun url(name: String, path: String): String = "http://$name:${server.localPort}$path"

    override fun close() = server.close()
}
