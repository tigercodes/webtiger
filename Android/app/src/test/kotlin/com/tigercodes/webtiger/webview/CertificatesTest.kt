// The certificates a person went on past (026): what is let through and for whom, whose
// certificate an address presents, and how a certificate is told from another.
package com.tigercodes.webtiger.webview

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val HOME = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val WORK = "7c2e4f6a-8b1d-4c3e-a5f7-9d0b2c4e6f8a"

class CertificatesTest {
    private val certificates = Certificates()
    private val router = Refused("https://router.example.com:443", "aa11")

    @Test
    fun nothingIsLetThroughUntilAWebViewIsToldTo() {
        assertFalse(certificates.trusts(HOME, router))
        // Refusing one, and holding it, lets nothing through.
        certificates.refused(4, router)
        assertFalse(certificates.trusts(HOME, router))
        // A web view that refused nothing holds nothing.
        assertFalse(certificates.trust(5, HOME))
        assertFalse(certificates.trusts(HOME, router))
    }

    @Test
    fun whatAWebViewLastRefusedIsLetThroughInItsLoginSetAndNoOther() {
        certificates.refused(4, router)

        assertTrue(certificates.trust(4, HOME))

        assertTrue(certificates.trusts(HOME, router))
        assertFalse("another login set is asked for itself", certificates.trusts(WORK, router))
        // That certificate at that host and port, and no other.
        assertFalse("another certificate at the same host", certificates.trusts(HOME, router.copy(fingerprint = "bb22")))
        assertFalse("the same certificate at another host", certificates.trusts(HOME, router.copy(origin = "https://printer.example.com:443")))
        assertFalse("the same certificate at another port", certificates.trusts(HOME, router.copy(origin = "https://router.example.com:8443")))
    }

    @Test
    fun aWebViewHoldsTheLastItRefusedAndNothingOnceItIsGone() {
        val printer = Refused("https://printer.example.com:443", "cc33")
        certificates.refused(4, router)
        certificates.refused(4, printer)
        assertTrue(certificates.trust(4, HOME))
        assertTrue(certificates.trusts(HOME, printer))
        assertFalse("only the last it refused", certificates.trusts(HOME, router))

        // Gone, it holds nothing; what was let through stays, for its login set.
        certificates.dropped(4)
        assertFalse(certificates.trust(4, WORK))
        assertTrue(certificates.trusts(HOME, printer))
        // And another web view's is its own.
        certificates.refused(6, router)
        certificates.dropped(4)
        assertTrue(certificates.trust(6, WORK))
        assertTrue(certificates.trusts(WORK, router))
    }

    @Test
    fun whoseCertificateAnAddressPresentsIsItsHostAndPortOverHttps() {
        assertEquals("https://router.example.com:443", certificateOrigin("https://router.example.com/"))
        assertEquals("https://router.example.com:443", certificateOrigin("https://Router.Example.com:443/status?x=1#top"))
        assertEquals("https://127.0.0.1:8443", certificateOrigin("https://127.0.0.1:8443/"))
        assertEquals("https://router.example.com:443", certificateOrigin("HTTPS://router.example.com"))
        // No certificate is presented at any of these.
        for (other in listOf("http://router.example.com/", "wss://router.example.com/", "about:blank", "", "not an address", "https://", "https:///path")) {
            assertNull(other, certificateOrigin(other))
        }
    }

    @Test
    fun aCertificateIsToldFromAnotherByAHashOfItself() {
        val one = certificateFingerprint(byteArrayOf(1, 2, 3))
        assertEquals(one, certificateFingerprint(byteArrayOf(1, 2, 3)))
        assertNotEquals(one, certificateFingerprint(byteArrayOf(1, 2, 4)))
        // SHA-256, in hex.
        assertEquals("039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81", one)
    }
}
