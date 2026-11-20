// The adapter suite's System WebView run, for a secure connection that cannot be made (021): a
// server that speaks no TLS, and one whose certificate signs itself. What the adapter says the
// trouble was, and that nothing of the site is shown.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.SelfSignedSite
import org.junit.Assert.assertEquals
import org.junit.Test

internal class SystemWebViewCertificateTest : AdapterTestCase() {
    /** The codes a web view's loads failed with, in order. */
    private fun failures(id: Long = 1): List<String> = reports(id).mapNotNull { it.navigation_failed?.code }

    @Test
    fun aServerThatSpeaksNoTlsAskedForOverHttpsIsASecureConnectionThatCouldNotBeMade() {
        create(1)
        // The test site speaks plain HTTP: asked for over https, there is nothing to trust.
        load(1, "https://127.0.0.1:${site.port}/")

        await(1, "the load to fail") { failed(it) }

        assertEquals(listOf("net.tls"), failures())
        assertEquals("nothing of the site was shown", emptyList<String>(), addresses(1))
    }

    @Test
    fun aCertificateThatSignsItselfIsNotTrustedAndThePageIsNeverAskedFor() {
        SelfSignedSite().use { secure ->
            create(1)
            load(1, "https://127.0.0.1:${secure.port}/")

            await(1, "the load to fail") { failed(it) }

            assertEquals("which kind of trouble the certificate was (the site's side ended: ${secure.lastEnd})", listOf("net.tls.untrusted"), failures())
            assertEquals("nothing of the site was shown", emptyList<String>(), addresses(1))
            // Nothing was accepted on anybody's behalf: the site was never asked for its page.
            Thread.sleep(500)
            assertEquals(0, secure.served.get())
        }
    }
}
