// The adapter suite's System WebView run, for a secure connection that cannot be made (021): a
// server that speaks no TLS, and one whose certificate signs itself. What the adapter says the
// trouble was, and that nothing of the site is shown.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.SelfSignedSite
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewCertificateTest : AdapterTestCase() {
    /** The codes a web view's loads failed with, in order. */
    private fun failures(id: Long = 1): List<String> = reports(id).mapNotNull { it.navigation_failed?.code }

    /**
     * No page of the site came of a load that failed: once the engine has had the time to show
     * its own error page, which it commits at the address that failed, the load has not
     * finished, no title is the site's, and no other address was named.
     */
    private fun assertNoPageCameOf(address: String) {
        Thread.sleep(1_000)
        assertFalse("a failed load never finishes", names(reports(1)).contains("finished"))
        assertFalse("the engine's error page has no title of the site's", names(reports(1)).contains("title"))
        assertTrue("the only address named is the one that failed: ${addresses(1)}", addresses(1).all { it == address })
    }

    @Test
    fun aServerThatSpeaksNoTlsAskedForOverHttpsIsASecureConnectionThatCouldNotBeMade() {
        create(1)
        // The test site speaks plain HTTP: asked for over https, there is nothing to trust.
        val address = "https://127.0.0.1:${site.port}/"
        load(1, address)

        await(1, "the load to fail") { failed(it) }

        assertNoPageCameOf(address)
        assertEquals("the failure is reported once, for what it was", listOf("net.tls"), failures())
    }

    @Test
    fun aCertificateThatSignsItselfIsNotTrustedAndThePageIsNeverAskedFor() {
        SelfSignedSite().use { secure ->
            create(1)
            val address = "https://127.0.0.1:${secure.port}/"
            load(1, address)

            await(1, "the load to fail") { failed(it) }

            assertNoPageCameOf(address)
            assertEquals("which kind of trouble the certificate was (the site's side ended: ${secure.lastEnd})", listOf("net.tls.untrusted"), failures())
            // Nothing was accepted on anybody's behalf: the site was never asked for its page.
            assertEquals(0, secure.served.get())
        }
    }
}
