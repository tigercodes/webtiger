// The adapter suite's System WebView run, for a secure connection that cannot be made (021): a
// server that speaks no TLS, and one whose certificate signs itself. What the adapter says the
// trouble was, and that nothing of the site is shown.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.SelfSignedSite
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.WebViewTrustCertificate
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
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
            // The adapter holds what it refused (026): it can be told to let it through.
            assertEquals(listOf(true), held())
        }
    }

    /** Whether each failure of a web view's loads said the adapter holds the certificate it refused, in order. */
    private fun held(id: Long = 1): List<Boolean> = reports(id).mapNotNull { it.navigation_failed?.certificate_held }

    private fun trust(id: Long): HostResponse = answer(HostRequest(web_view_trust_certificate = WebViewTrustCertificate(web_view_id = id)))

    /**
     * Going on past a certificate (026), against the real WebView: nothing is let through until
     * the adapter is told; told, the certificate it refused is let through and the page loads;
     * and it is let through for the login set it was told for, and no other.
     */
    @Test
    fun aCertificateIsLetThroughWhenTheAdapterIsToldAndOnlyInThatLoginSet() {
        SelfSignedSite().use { secure ->
            val address = "https://127.0.0.1:${secure.port}/"
            val home = newLoginSet()
            create(1, loginSet = home)
            // A web view that refused nothing holds nothing.
            assertEquals("certificate.unknown", trust(1).error?.code)
            load(1, address)
            await(1, "the load to fail") { failed(it) }
            assertEquals(0, secure.served.get())

            assertNull(trust(1).error)
            load(1, address)

            await(1, "the site's page") { hasTitle("Trusted after all")(it) && finished(it) }
            assertEquals("the site was asked for its page, once", 1, secure.served.get())
            assertEquals("no failure but the first", listOf("net.tls.untrusted"), failures(1))

            // Another web view of the same login set: let through already.
            create(2, loginSet = home)
            load(2, address)
            await(2, "the site's page in the same login set") { hasTitle("Trusted after all")(it) && finished(it) }
            assertEquals(emptyList<String>(), failures(2))

            // Another login set is asked for itself: refused, held, and the site not asked.
            val asked = secure.served.get()
            create(3, loginSet = newLoginSet())
            load(3, address)
            await(3, "the load to fail in another login set") { failed(it) }
            assertEquals(listOf("net.tls.untrusted"), failures(3))
            assertEquals(listOf(true), held(3))
            Thread.sleep(500)
            assertEquals("nothing was let through for it", asked, secure.served.get())
        }
    }

    /**
     * One certificate and no other (026 §3): the same host and port with another certificate is
     * refused again, though a person went on past the one before.
     */
    @Test
    fun anotherCertificateAtTheSameHostAndPortIsRefusedAgain() {
        val first = SelfSignedSite()
        val port = first.port
        val address = "https://127.0.0.1:$port/"
        try {
            create(1)
            load(1, address)
            await(1, "the load to fail") { failed(it) }
            assertNull(trust(1).error)
            load(1, address)
            await(1, "the site's page") { hasTitle("Trusted after all")(it) && finished(it) }
        } finally {
            first.close()
        }

        // The same place, with a key and a certificate made anew.
        SelfSignedSite(port).use { second ->
            load(1, address)
            await(1, "the second load to fail") { reports -> reports.count { it.navigation_failed != null } >= 2 }
            assertEquals(listOf("net.tls.untrusted", "net.tls.untrusted"), failures(1))
            assertEquals("held, to be gone on with afresh", listOf(true, true), held(1))
            Thread.sleep(500)
            assertEquals("its page was never asked for", 0, second.served.get())

            // Gone on with again, it is this one that is let through.
            assertNull(trust(1).error)
            load(1, address)
            await(1, "the site's page again") { reports -> reports.count { it.navigation_finished != null } >= 2 }
            assertEquals(1, second.served.get())
        }
    }
}
