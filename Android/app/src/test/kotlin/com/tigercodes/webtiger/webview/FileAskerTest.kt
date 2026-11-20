// Where a file the person chose is (022): the walk from its address to the file, with no site
// behind it. Who is asked, with whose cookies, and where the asking stops.
package com.tigercodes.webtiger.webview

import org.junit.Assert.assertEquals
import org.junit.Test

private const val HERE = "https://example.com/files/report"
private const val THERE = "https://cdn.example.net/2026/report.pdf"

class FileAskerTest {
    /** Who was asked, in order: the address, and the cookies it was asked with. */
    private val asked = ArrayList<Pair<String, String?>>()

    /** What each site knows the person by. */
    private val cookies = mapOf("example.com" to "who=ada", "cdn.example.net" to "seen=before")

    private fun cookiesFor(address: String): String? = cookies.entries.firstOrNull { address.contains("//${it.key}/") }?.value

    private fun walk(url: String, answers: Map<String, Answer?>): Told =
        whereTheFileIs(
            url,
            cookiesFor = ::cookiesFor,
            ask = { address, with ->
                asked.add(address to with)
                answers[address]
            },
            name = { _, disposition, _ -> disposition.substringAfter("filename=").trim('"') },
        )

    @Test
    fun aFileWhereItsAddressSaysIsToldWithWhatItsSiteSaysOfIt() {
        val told = walk(HERE, mapOf(HERE to Answer(200, mimeType = "application/pdf", disposition = "attachment; filename=\"Report 2026.pdf\"", length = 4096)))

        assertEquals(Told(HERE, "application/pdf", "Report 2026.pdf", 4096), told)
        assertEquals("asked once, as the site knows the person", listOf(HERE to "who=ada"), asked)
    }

    @Test
    fun aSiteThatSaysNoNameSuggestsNoneAndOneThatSaysNoSizeIsNone() {
        val told = walk(HERE, mapOf(HERE to Answer(200, mimeType = "image/png", disposition = " ", length = -1)))

        assertEquals(Told(HERE, "image/png", "", 0), told)
    }

    @Test
    fun aFileThatIsSomewhereElseIsAskedAfterThereWithThatSitesCookiesAndNoOthers() {
        val told =
            walk(
                HERE,
                mapOf(
                    HERE to Answer(302, location = THERE, mimeType = "text/html"),
                    THERE to Answer(200, mimeType = "application/pdf", length = 4096),
                ),
            )

        // The file is where the first site sent whoever asked, and it is that site's file.
        assertEquals(Told(THERE, "application/pdf", "", 4096), told)
        // What the first site knows the person by was not sent to the second.
        assertEquals(listOf(HERE to "who=ada", THERE to "seen=before"), asked)
    }

    @Test
    fun aSiteThatSendsOnWithOnlyAPathSendsOnWithinItself() {
        val moved = "https://example.com/archive/report.pdf"
        val told = walk(HERE, mapOf(HERE to Answer(301, location = "/archive/report.pdf"), moved to Answer(200, mimeType = "application/pdf")))

        assertEquals(Told(moved, "application/pdf"), told)
        assertEquals(listOf(HERE to "who=ada", moved to "who=ada"), asked)
    }

    @Test
    fun theAskingStopsWhereNobodyAnswersOrTheAnswerIsNoFiles() {
        // Nobody there: the file is taken to be where its address says, with nothing known.
        assertEquals(Told(HERE), walk(HERE, emptyMap()))
        // Not found, and an answer that sends on without saying where.
        assertEquals(Told(HERE), walk(HERE, mapOf(HERE to Answer(404, mimeType = "text/html", length = 120))))
        assertEquals(Told(HERE), walk(HERE, mapOf(HERE to Answer(302, mimeType = "text/html"))))
        // Sent on to somewhere nobody answers: that is where it is, with nothing known.
        assertEquals(Told(THERE), walk(HERE, mapOf(HERE to Answer(302, location = THERE))))
    }

    @Test
    fun anAddressThatIsNotTheWebsIsNotFollowed() {
        for (elsewhere in listOf("file:///etc/hosts", "intent://example.com#Intent;end", "javascript:alert(1)", "data:text/plain,hello")) {
            asked.clear()
            val told = walk(HERE, mapOf(HERE to Answer(302, location = elsewhere)))

            assertEquals(elsewhere, Told(HERE), told)
            assertEquals(elsewhere, listOf<Pair<String, String?>>(HERE to "who=ada"), asked)
        }
    }

    @Test
    fun anAddressThatSendsOnForEverIsFollowedOnlySoFar() {
        // Each address sends on to the next.
        val round = (0..MOST_HOPS + 5).associate { "https://example.com/$it" to Answer(302, location = "https://example.com/${it + 1}") }

        val told = walk("https://example.com/0", round)

        assertEquals(MOST_HOPS, asked.size)
        assertEquals(Told("https://example.com/$MOST_HOPS"), told)
    }
}
