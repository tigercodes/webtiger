// A site over https whose certificate signs itself: one no device trusts. For the tests of what
// an engine is told, and tells, when a certificate cannot be trusted (021). The key and its
// certificate are made on the device, in the app's own key store, when a test asks for the
// site, and are removed with it: no key is kept in the repository.
package com.tigercodes.webtiger

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.Closeable
import java.net.InetAddress
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.spec.ECGenParameterSpec
import java.util.concurrent.atomic.AtomicInteger
import javax.net.ssl.KeyManagerFactory
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLServerSocket
import javax.net.ssl.SSLSocket
import javax.security.auth.x500.X500Principal
import kotlin.concurrent.thread

internal class SelfSignedSite : Closeable {
    private val alias = "webtiger-test-site-${System.nanoTime()}"
    private val store = KeyStore.getInstance(STORE).apply { load(null) }
    private val server: SSLServerSocket

    /** How many requests for a page were answered: none, when the browser would not trust the certificate. */
    val served = AtomicInteger(0)

    /** How the last connection ended, when it ended badly: for a test that has to say why. */
    @Volatile var lastEnd: String = ""

    /** The port the site listens on. */
    val port: Int get() = server.localPort

    init {
        // An elliptic-curve key: the system's TLS signs a handshake with one that stays in the
        // key store. (With an RSA key of the key store's, it could not: the handshake ended
        // on the site's side before the certificate was sent.)
        KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, STORE).apply {
            initialize(
                KeyGenParameterSpec
                    .Builder(alias, KeyProperties.PURPOSE_SIGN)
                    .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
                    .setCertificateSubject(X500Principal("CN=127.0.0.1"))
                    // What a TLS handshake signs with, in either of its versions.
                    .setDigests(KeyProperties.DIGEST_NONE, KeyProperties.DIGEST_SHA256, KeyProperties.DIGEST_SHA384, KeyProperties.DIGEST_SHA512)
                    .build(),
            )
            generateKeyPair()
        }
        val keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm()).apply { init(store, null) }
        val tls = SSLContext.getInstance("TLS").apply { init(keys.keyManagers, null, null) }
        server = tls.serverSocketFactory.createServerSocket(0, 50, InetAddress.getByName("127.0.0.1")) as SSLServerSocket
        thread(isDaemon = true, name = "webtiger-self-signed-site") { serve() }
    }

    private fun serve() {
        while (!server.isClosed) {
            val socket = try {
                server.accept() as SSLSocket
            } catch (_: java.io.IOException) {
                return
            }
            thread(isDaemon = true) {
                try {
                    socket.use {
                        it.soTimeout = 5_000
                        // A browser that does not trust the certificate ends the connection here.
                        it.startHandshake()
                        val input = it.inputStream
                        val received = StringBuilder()
                        val buffer = ByteArray(4096)
                        while (!received.contains("\r\n\r\n")) {
                            val count = input.read(buffer)
                            if (count < 0) return@use
                            received.append(String(buffer, 0, count, Charsets.ISO_8859_1))
                        }
                        served.incrementAndGet()
                        val body = "<!doctype html><title>Trusted after all</title>".toByteArray()
                        it.outputStream.apply {
                            write("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: ${body.size}\r\nConnection: close\r\n\r\n".toByteArray())
                            write(body)
                            flush()
                        }
                    }
                } catch (ended: java.io.IOException) {
                    // The browser went away, as it does from a certificate it does not trust.
                    lastEnd = ended.toString() + " / " + ended.cause
                }
            }
        }
    }

    override fun close() {
        server.close()
        store.deleteEntry(alias)
    }

    private companion object {
        const val STORE = "AndroidKeyStore"
    }
}
