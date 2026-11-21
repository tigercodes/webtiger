// The adapter suite's System WebView run, for downloads (013): what the adapter reports when
// a page offers a file, and what is fetched, where and under what name, once the core has
// said. The WebView does not download: the adapter fetches the address again itself, each
// address on the way with the login set's cookies for it and no other's (027). The files go to
// the app's own folder for downloads, which is emptied around each test; never to the device's.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.os.Environment
import android.webkit.WebView
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.core.v1.DownloadCancel
import com.tigercodes.webtiger.core.v1.DownloadRequested
import com.tigercodes.webtiger.core.v1.DownloadStart
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.io.File

internal class SystemWebViewDownloadTest : AdapterTestCase() {
    private val folder: File = checkNotNull(ApplicationProvider.getApplicationContext<Context>().getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS))

    @Before
    @After
    fun emptyTheFolder() {
        folder.listFiles()?.forEach { it.delete() }
    }

    /** A web view on the page of links that each offer a file a different way. */
    private fun page(id: Long = 1) {
        create(id)
        load(id, site.url("/downloads"))
        await(id, "the page to load") { finished(it) && hasTitle("Downloads")(it) }
    }

    /** The downloads the adapter has reported a web view's page offering, in order. */
    private fun asked(id: Long = 1): List<DownloadRequested> = reports(id).mapNotNull { it.download_requested }

    /** The names the adapter has said downloads finished under, in order. */
    private fun finished(id: Long = 1): List<String> = reports(id).mapNotNull { it.download_finished?.file_name }

    /** What the adapter has said of downloads that failed, in order. */
    private fun failed(id: Long = 1): List<String> = reports(id).mapNotNull { it.download_failed?.code }

    /** Taps a link of the page with a finger, and waits for the download it offers to be reported. */
    private fun tapLink(link: String, id: Long = 1): DownloadRequested {
        val before = asked(id).size
        tap(id, link)
        await(id, "a download to be asked for") { asked(id).size > before }
        return asked(id).last()
    }

    private fun start(download: Long, name: String): HostResponse = answer(HostRequest(download_start = DownloadStart(download_id = download, file_name = name)))

    /** Starts a download and waits for its file to be whole: up to a minute, the system being no hurry. The name it finished under. */
    private fun save(download: DownloadRequested, name: String, id: Long = 1): String {
        val before = finished(id).size
        assertNotNull("a start is acknowledged", start(download.download_id, name).ack)
        repeat(600) {
            if (finished(id).size > before) return finished(id).last()
            check(failed(id).isEmpty()) { "the download failed: ${failed(id)}" }
            Thread.sleep(100)
        }
        throw AssertionError("the download never finished; reported: ${names(reports(id))}")
    }

    /** What is in the file of that name in the folder. Null when there is none. */
    private fun saved(name: String): String? = File(folder, name).takeIf { it.isFile }?.readText()

    private fun names(): List<String> = folder.listFiles().orEmpty().map { it.name }.sorted()

    @Test
    fun anAttachmentIsReportedAndFetchedUnderTheNameTheCoreGives() {
        page()

        val download = tapLink("attachment")

        assertEquals(site.url("/notes"), download.url)
        assertEquals("what the site suggests, as the platform reads it", "notes.txt", download.suggested_filename)
        assertEquals("text/plain", download.mime_type)
        assertEquals(26L, download.expected_bytes)
        assertNotEquals("named by the shell", 0L, download.download_id)
        assertEquals("nothing is fetched until the core says under what name", emptyList<String>(), names())

        val name = save(download, "My notes.txt")

        assertEquals("My notes.txt", name)
        assertEquals("Twelve notes, to be saved.", saved("My notes.txt"))
        assertEquals(listOf("My notes.txt"), names())
        // The page that offered it is the page on screen: the load that became a download stopped.
        assertEquals("Downloads", lastTitle(1))
        assertTrue("no load failed but by stopping: ${names(reports(1))}", names(reports(1)).none { it.startsWith("failed(") && it != "failed(net.cancelled)" })
    }

    @Test
    fun aKindNothingShowsIsADownload() {
        page()

        val download = tapLink("kind")

        assertEquals(site.url("/data.bin"), download.url)
        assertEquals(16L, download.expected_bytes)
        assertEquals("0123456789abcdef", saved(save(download, "data.bin")))
    }

    @Test
    fun aLinkThatSaysDownloadIsOne() {
        page()

        // A page a browser would show: the link says it is to be saved, and under what name.
        val download = tapLink("says")

        assertEquals(site.url("/landed"), download.url)
        // The WebView says a download was asked for, and not what the link said to call it:
        // the name suggested is made from the address and the kind.
        assertEquals("landed.html", download.suggested_filename)
        assertEquals("text/html", download.mime_type)
        val name = save(download, "kept page.html")
        assertEquals("kept page.html", name)
        assertTrue(saved(name).orEmpty().contains("<title>Landed</title>"))
        assertEquals("and the web view went nowhere", site.url("/downloads"), onMain { (engine.webView(1) as? WebView)?.url })
    }

    @Test
    fun aDownloadIsFetchedAsItsLoginSet() {
        // Signed in, in this web view's login set; then a file that says who asked for it. The
        // system fetches the address in a process of its own: the cookies are what the adapter
        // hands it, and they are the login set's.
        create(1)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/downloads"))
        await(1, "the page of links") { hasTitle("Downloads")(it) && finished(it, times = 2) }

        assertEquals("who=ada", saved(save(tapLink("whose"), "who.txt")))

        // Another login set's web view is nobody there.
        page(2)
        assertEquals("who=none", saved(save(tapLink("whose", id = 2), "who else.txt", id = 2)))
    }

    @Test
    fun aFileThePersonChoseThatIsSomewhereElseIsNotFetchedThereAsThisSite() {
        // Signed in at this site, in this web view's login set.
        create(1)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }

        // A file of this site, chosen in a menu (022), that says who asked for it: fetched as
        // the login set.
        onMain { engine.saveFile(1, site.url("/whose-file")) }
        await(1, "the file to be offered") { reports -> reports.count { it.download_requested != null } == 1 }
        assertEquals(site.url("/whose-file"), asked().last().url)
        assertEquals("the person chose it", true, asked().last().user_asked)
        assertEquals("who=ada", saved(save(asked().last(), "here.txt")))

        // The same file, at an address of this site that sends whoever fetches it on to
        // another site (the server by its other name): what this site knows the person by is
        // not sent there.
        onMain { engine.saveFile(1, site.url("/to-whose-file?port=${site.port}")) }
        await(1, "the file to be offered") { reports -> reports.count { it.download_requested != null } == 2 }
        // Offered as where the file is, with what the site there says of it.
        assertEquals("http://localhost:${site.port}/whose-file", asked().last().url)
        assertEquals("who.txt", asked().last().suggested_filename)
        assertEquals("text/plain", asked().last().mime_type)
        assertEquals("who=none", saved(save(asked().last(), "elsewhere.txt")))
    }

    /**
     * Cookies stay with their site (027), for a download a page starts: a link of this site
     * leads to a file that is at another site (the server by its other name). The person is
     * nobody there, and is not made somebody by what this site knows them by; signed in there
     * too, as somebody else, the file is fetched as that site knows them.
     */
    @Test
    fun aDownloadAPageStartsThatIsAtAnotherSiteIsFetchedThereAsThatSiteKnowsThePerson() {
        create(1)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/downloads-elsewhere?port=${site.port}"))
        await(1, "the page of links") { hasTitle("Downloads elsewhere")(it) }

        val download = tapLink("elsewhere")

        assertEquals("http://localhost:${site.port}/whose-file", download.url)
        assertEquals("who=none", saved(save(download, "elsewhere.txt")))

        // Signed in at the other site as somebody else, in the same login set.
        load(1, "http://localhost:${site.port}/set?v=grace")
        await(1, "the other site's cookie to be set") { reports -> reports.count { it.title_changed?.title == "Set" } == 2 }
        load(1, site.url("/downloads-elsewhere?port=${site.port}"))
        await(1, "the page of links again") { reports -> reports.count { it.title_changed?.title == "Downloads elsewhere" } == 2 }
        assertEquals("who=grace", saved(save(tapLink("elsewhere"), "theirs.txt")))
        // And this site still knows them as it did.
        load(1, site.url("/whoami"))
        await(1, "who this site says is signed in") { hasTitle("who=ada")(it) }
    }

    /**
     * The fetcher itself (027), with the WebView's own cookie store: given an address of this
     * site that sends it on to another site, it follows by hand, and what this site knows the
     * person by is not sent there. (The system's downloader, which the adapter used before,
     * sent it: the file said "who=ada".) A cookie a site sets on the way is kept for that site,
     * in the login set: the file is fetched with it, and a page of that site then sees it.
     */
    @Test
    fun theFetcherFollowsWhoeverSendsOnWithEachSitesOwnCookies() {
        val loginSet = newLoginSet()
        create(1, loginSet)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        val fetcher = AppFetcher(ApplicationProvider.getApplicationContext<Context>(), DownloadPlace.APP)

        fun fetch(address: String, name: String): String? {
            val handle = checkNotNull(onMain { fetcher.start(DownloadOffer(address, "", "", "", 0), loginSet, name) })
            repeat(600) {
                when (val status = fetcher.status(handle)) {
                    is FetchStatus.Finished -> return saved(status.fileName)
                    is FetchStatus.Coming -> Thread.sleep(50)
                    else -> error("the fetch ended as $status")
                }
            }
            error("the fetch never ended")
        }

        assertEquals("this site's file, as this site knows the person", "who=ada", fetch(site.url("/whose-file"), "here.txt"))
        assertEquals("the other site's file, as nobody", "who=none", fetch(site.url("/to-whose-file?port=${site.port}"), "followed.txt"))

        // A cookie set on the way: the file is fetched with it, and the login set keeps it.
        assertEquals("who=lin", fetch(site.url("/signs-in-and-sends?v=lin"), "signed.txt"))
        load(1, site.url("/whoami"))
        await(1, "who this site now says is signed in") { hasTitle("who=lin")(it) }
        // A web view of another login set is nobody still.
        create(2)
        load(2, site.url("/whoami"))
        await(2, "who another login set is") { hasTitle("who=none")(it) }
    }

    @Test
    fun aDownloadTheCoreDropsFetchesNothing() {
        page()
        val download = tapLink("attachment")

        assertNotNull("dropping is acknowledged", answer(HostRequest(download_cancel = DownloadCancel(download_id = download.download_id))).ack)
        Thread.sleep(1_500)

        assertEquals(emptyList<String>(), names())
        assertEquals(emptyList<String>(), finished())
        // It is no longer waiting for a word: one that comes now is for nothing.
        assertEquals("download.unknown", start(download.download_id, "late.txt").error?.code)
        Thread.sleep(500)
        assertEquals(emptyList<String>(), names())
    }

    @Test
    fun aNameThatIsTakenIsNotWrittenOver() {
        page()
        File(folder, "notes.txt").writeText("mine")

        val name = save(tapLink("attachment"), "notes.txt")

        assertNotEquals("the system chose another name, and the adapter says which", "notes.txt", name)
        assertEquals("what was there is as it was", "mine", saved("notes.txt"))
        assertEquals("Twelve notes, to be saved.", saved(name))
    }

    @Test
    fun aNameIsHeldToANameWhateverItSays() {
        page()
        // The site's own suggestion has folders in it. The core makes the name (013 §4); a core
        // that gave a path all the same is held to its last part.
        val download = tapLink("sneaky")

        val name = save(download, "../../escaped.txt")

        assertEquals("escaped.txt", name)
        assertEquals(listOf("escaped.txt"), names())
    }

    @Test
    fun aWordAboutADownloadThereIsNot() {
        assertEquals("download.unknown", start(99, "nothing.txt").error?.code)
        assertNotNull("dropping what is not there is acknowledged", answer(HostRequest(download_cancel = DownloadCancel(download_id = 99))).ack)
    }
}
