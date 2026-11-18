// A web site on the device's own loopback address, for the tests that run there: pages that set
// and read a cookie, redirect, and change their own address. The same pages, at the same paths,
// as the Apple suite's site (Packages/Tests/WTWebKitTests/TestSite.swift); Tools/testsite will
// give every shell's suite one definition of them.
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

    private fun respond(head: String): ByteArray {
        val lines = head.split("\r\n")
        val target = lines.first().split(" ").getOrElse(1) { "/" }
        val path = target.substringBefore("?")
        val query = target.substringAfter("?", "")
        asked.add(path)
        val cookies = lines.firstOrNull { it.lowercase().startsWith("cookie:") }?.substring("cookie:".length)?.trim().orEmpty()
        val who = cookies.split(";").map { it.trim() }.firstOrNull { it.startsWith("who=") }?.removePrefix("who=")

        return when (path) {
            "/" -> page("Home", body = "<a id=\"next\" href=\"/landed\">next</a>")
            "/set" -> {
                // ?v=<value>: remembers it in a cookie.
                val value = query.split("&").firstOrNull { it.startsWith("v=") }?.removePrefix("v=") ?: "someone"
                page("Set", headers = listOf("Set-Cookie: who=$value; Path=/; Max-Age=3600"))
            }
            "/whoami" -> page(if (who != null) "who=$who" else "who=none")
            "/redirect" -> response("302 Found", listOf("Location: /landed"), "")
            "/landed" -> page("Landed")
            "/push" -> page("Push", body = "<script>history.pushState({}, '', '/pushed')</script>")
            // A service worker that does nothing but exist.
            "/worker.js" -> response("200 OK", emptyList(), "self.addEventListener('install', () => self.skipWaiting());", type = "text/javascript")
            // Taller than any window, with a field to type in: for scroll and form restoration.
            "/tall" -> page("Tall", body = "<input id=\"note\"><div style=\"height: 6000px\"></div><p id=\"end\">end</p>")
            // Remembers ?v= in localStorage, or says what was remembered.
            "/storage" ->
                page(
                    "Storage",
                    body =
                        """
                        <script>
                        const v = new URLSearchParams(location.search).get('v');
                        if (v) localStorage.setItem('who', v);
                        document.title = 'stored=' + (localStorage.getItem('who') || 'none');
                        </script>
                        """.trimIndent(),
                )
            // A page that fetches something of its own: for the request callback.
            "/tracked" -> page("Tracked", body = "<img src=\"/pixel\" alt=\"\">")
            "/pixel" -> response("200 OK", emptyList(), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1\" height=\"1\"/>", type = "image/svg+xml")
            // Answers after a while: a load that is still going when a test stops it.
            "/slow" -> {
                Thread.sleep(3_000)
                page("Slow")
            }
            else -> response("404 Not Found", emptyList(), "<title>Not found</title>")
        }
    }

    private fun page(title: String, headers: List<String> = emptyList(), body: String = ""): ByteArray =
        response("200 OK", headers, "<!doctype html><title>$title</title><h1>$title</h1>$body")

    private fun response(status: String, headers: List<String>, body: String, type: String = "text/html"): ByteArray {
        val bytes = body.toByteArray(Charsets.UTF_8)
        val head =
            listOf(
                "HTTP/1.1 $status",
                "Content-Type: $type; charset=utf-8",
                "Content-Length: ${bytes.size}",
                "Cache-Control: no-store",
                "Connection: close",
            ) + headers
        return (head.joinToString("\r\n") + "\r\n\r\n").toByteArray(Charsets.ISO_8859_1) + bytes
    }

    companion object {
        /** A loopback port nothing listens on: one the system just handed out and took back. */
        fun unusedPort(): Int = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1")).use { it.localPort }
    }
}
