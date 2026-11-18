// The test site on the device's own loopback address, for the tests that run there. Its pages
// are defined once for every shell, in Fixtures/testsite/site.json, and written into
// TestSitePages.kt by Tools/testsite/site.py; this file only serves them.
//
// It lives in the test's process, so a suite needs nothing running beside it, and it is on
// 127.0.0.1 so that its pages are a secure context: the Cache API and service workers exist
// only there.
package com.tigercodes.webtiger

import java.io.Closeable
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketException
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.Executors

/** A piece of a header or a body: words as they are, or a value the request brought. */
internal sealed interface TestSitePiece {
    data class Text(val text: String) : TestSitePiece

    /** The request's query value of this name. */
    data class Query(val name: String, val or: String) : TestSitePiece

    /** The request's cookie of this name. */
    data class Cookie(val name: String, val or: String) : TestSitePiece
}

/** One page of the site: what a request for its path is answered with. */
internal data class TestSitePage(
    val path: String,
    val status: String,
    val type: String,
    val headers: List<List<TestSitePiece>>,
    val body: List<TestSitePiece>,
    val delayMilliseconds: Int,
)

internal class TestSite : Closeable {
    private val server = ServerSocket(0, 50, InetAddress.getByName("127.0.0.1"))
    private val workers = Executors.newCachedThreadPool { work -> Thread(work, "webtiger-testsite").apply { isDaemon = true } }
    private val asked = ConcurrentLinkedQueue<String>()

    /** The port the site listens on. */
    val port: Int get() = server.localPort

    /** Every path asked for, in order, without its query. */
    val requests: List<String> get() = asked.toList()

    init {
        workers.execute {
            while (!server.isClosed) {
                val connection =
                    try {
                        server.accept()
                    } catch (_: SocketException) {
                        break
                    }
                workers.execute { serve(connection) }
            }
        }
    }

    /** The address of a path on the site. */
    fun url(path: String): String = "http://127.0.0.1:$port$path"

    override fun close() {
        server.close()
        workers.shutdownNow()
    }

    private fun serve(connection: Socket) {
        connection.use { socket ->
            try {
                socket.soTimeout = 5_000
                val head = readHead(socket) ?: return
                socket.getOutputStream().apply {
                    write(respond(head))
                    flush()
                }
            } catch (_: java.io.IOException) {
                // The web view went away: nobody is left to answer.
            }
        }
    }

    /** A request's head, up to the empty line that ends it. */
    private fun readHead(socket: Socket): String? {
        val input = socket.getInputStream()
        val received = StringBuilder()
        val buffer = ByteArray(4096)
        while (!received.contains("\r\n\r\n")) {
            val count = input.read(buffer)
            if (count < 0) return null
            received.append(String(buffer, 0, count, Charsets.ISO_8859_1))
        }
        return received.substring(0, received.indexOf("\r\n\r\n"))
    }

    /** The whole answer to a request's head, given after the page's delay. */
    private fun respond(head: String): ByteArray {
        val lines = head.split("\r\n")
        val target = lines.first().split(" ").getOrElse(1) { "/" }
        val path = target.substringBefore("?")
        asked.add(path)
        val query = pairs(target.substringAfter("?", ""), '&')
        val cookies = pairs(lines.drop(1).firstOrNull { it.lowercase().startsWith("cookie:") }?.substring("cookie:".length).orEmpty(), ';')

        val page = TestSitePages.all.firstOrNull { it.path == path } ?: TestSitePages.notFound
        if (page.delayMilliseconds > 0) Thread.sleep(page.delayMilliseconds.toLong())
        val body = filled(page.body, query, cookies).toByteArray(Charsets.UTF_8)
        val answer =
            listOf(
                "HTTP/1.1 ${page.status}",
                "Content-Type: ${page.type}; charset=utf-8",
                "Content-Length: ${body.size}",
                "Cache-Control: no-store",
                "Connection: close",
            ) + page.headers.map { filled(it, query, cookies) }
        return (answer.joinToString("\r\n") + "\r\n\r\n").toByteArray(Charsets.ISO_8859_1) + body
    }

    /** `name=value` pairs. The first of a name is the one kept. */
    private fun pairs(text: String, separator: Char): Map<String, String> {
        val found = LinkedHashMap<String, String>()
        for (pair in text.split(separator)) {
            val name = pair.trim().substringBefore("=")
            if (pair.contains("=") && name !in found) found[name] = pair.trim().substringAfter("=")
        }
        return found
    }

    private fun filled(pieces: List<TestSitePiece>, query: Map<String, String>, cookies: Map<String, String>): String =
        pieces.joinToString("") { piece ->
            when (piece) {
                is TestSitePiece.Text -> piece.text
                is TestSitePiece.Query -> query[piece.name] ?: piece.or
                is TestSitePiece.Cookie -> cookies[piece.name] ?: piece.or
            }
        }

    companion object {
        /** A loopback port nothing listens on: one the system just handed out and took back. */
        fun unusedPort(): Int = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1")).use { it.localPort }
    }
}
