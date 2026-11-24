// The WebViews host service, with pages that have no WebView behind them: what the engine
// answers, and what it refuses with which code.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostPing
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.NavigationFinished
import com.tigercodes.webtiger.core.v1.OpenOutside
import com.tigercodes.webtiger.core.v1.WebViewCommand
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import com.tigercodes.webtiger.core.v1.WebViewFind
import com.tigercodes.webtiger.core.v1.WebViewSetZoom
import com.tigercodes.webtiger.core.v1.WebViewLoad
import com.tigercodes.webtiger.core.v1.WebViewRestore
import com.tigercodes.webtiger.core.v1.WebViewSaveState
import com.tigercodes.webtiger.core.v1.WebViewTrustCertificate
import kotlinx.coroutines.test.runTest
import okio.ByteString.Companion.encodeUtf8
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val LOGIN_SET = "0f8b1c9e-2d47-4a6b-9c3e-5a1d7e8f9a0b"
private const val OTHER_LOGIN_SET = "7c2e4f6a-8b1d-4c3e-a5f7-9d0b2c4e6f8a"

class SystemWebViewEngineTest {
    private val pages = FakePages()
    private val core = ScriptedCore()
    private val engine = SystemWebViewEngine(pages).also { it.attach(core) }

    private fun create(id: Long, loginSet: LoginSet? = LoginSet(id = LOGIN_SET), kind: EngineKind = EngineKind.ENGINE_KIND_CHROMIUM) =
        HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = kind, login_set = loginSet))

    private fun load(id: Long, url: String) = HostRequest(web_view_load = WebViewLoad(web_view_id = id, url = url))

    private fun command(id: Long, command: NavigationCommand) = HostRequest(web_view_command = WebViewCommand(web_view_id = id, command = command))

    private fun destroy(id: Long) = HostRequest(web_view_destroy = WebViewDestroy(web_view_id = id))

    private fun assertDone(response: HostResponse) {
        assertNull("expected an Ack, got ${response.error?.code}", response.error)
        assertNotNull(response.ack)
    }

    private fun assertRefused(code: String, response: HostResponse) {
        assertEquals(code, response.error?.code)
        assertNull(response.ack)
    }

    private fun trust(id: Long) = HostRequest(web_view_trust_certificate = WebViewTrustCertificate(web_view_id = id))

    /**
     * Going on past a certificate (026): the engine lets through what a web view last refused,
     * in that web view's login set, and says when it holds nothing.
     */
    @Test
    fun aCertificateAWebViewRefusedIsLetThroughInItsLoginSetWhenTheCoreSays() =
        runTest {
            val router = Refused("https://router.example.com:443", "aa11")
            assertRefused("webview.unknown", engine.answer(trust(4)))
            assertDone(engine.answer(create(4)))
            assertDone(engine.answer(create(5, LoginSet(id = OTHER_LOGIN_SET))))
            // A web view that refused nothing holds nothing to let through.
            assertRefused("certificate.unknown", engine.answer(trust(4)))
            assertFalse(engine.certificates.trusts(LOGIN_SET, router))

            // Its page was refused the router's certificate, as the page would record it.
            engine.certificates.refused(4, router)
            assertFalse("held is not let through", engine.certificates.trusts(LOGIN_SET, router))
            assertDone(engine.answer(trust(4)))
            assertTrue(engine.certificates.trusts(LOGIN_SET, router))
            assertFalse("the other login set is asked for itself", engine.certificates.trusts(OTHER_LOGIN_SET, router))
            assertRefused("certificate.unknown", engine.answer(trust(5)))

            // A web view that is destroyed holds nothing; what was let through stays.
            engine.certificates.refused(5, router)
            assertDone(engine.answer(destroy(5)))
            assertRefused("webview.unknown", engine.answer(trust(5)))
            assertFalse(engine.certificates.trusts(OTHER_LOGIN_SET, router))
            assertTrue(engine.certificates.trusts(LOGIN_SET, router))
        }

    @Test
    fun thePingIsAnsweredWithItsPayload() =
        runTest {
            val response = engine.answer(HostRequest(ping = HostPing(payload = "hello".encodeUtf8())))
            assertEquals("hello".encodeUtf8(), response.pong?.payload)
        }

    @Test
    fun aWebViewIsMadeInItsLoginSet() =
        runTest {
            val before = engine.generation
            assertDone(engine.answer(create(4)))
            assertDone(engine.answer(create(5, LoginSet(id = OTHER_LOGIN_SET))))
            assertEquals(2, engine.webViewCount)
            assertEquals(listOf(4L to LOGIN_SET, 5L to OTHER_LOGIN_SET), pages.made.map { it.id to it.loginSet })
            assertTrue("a window showing a web view draws again when the web views change", engine.generation > before)
        }

    @Test
    fun aWebViewThatCannotBeMadeIsRefusedWithACode() =
        runTest {
            assertRefused("engine.unavailable", engine.answer(create(4, kind = EngineKind.ENGINE_KIND_WEBKIT)))
            assertRefused("webview.bad_login_set", engine.answer(create(4, loginSet = null)))
            assertRefused("webview.bad_login_set", engine.answer(create(4, LoginSet(id = LOGIN_SET.uppercase()))))
            assertRefused("webview.bad_login_set", engine.answer(create(4, LoginSet(id = "../Default"))))
            // A private login set that cannot be written down as one to throw away is not made:
            // a profile nobody knows to delete would be left on disk.
            pages.remembers = false
            assertRefused("engine.no_private", engine.answer(create(4, LoginSet(id = LOGIN_SET, ephemeral = true))))
            pages.remembers = true
            assertEquals(0, engine.webViewCount)
            assertTrue(pages.made.isEmpty())

            assertDone(engine.answer(create(4)))
            assertRefused("webview.exists", engine.answer(create(4)))
            assertEquals(1, pages.made.size)
        }

    @Test
    fun withoutProfilesNoWebViewIsMadeRatherThanOneThatSharesItsData() =
        runTest {
            pages.keepsLoginSetsApart = false
            assertRefused("engine.no_profiles", engine.answer(create(4)))
            assertEquals(0, engine.webViewCount)
            assertTrue("no web view was made in the default profile", pages.made.isEmpty())
            assertRefused("webview.unknown", engine.answer(load(4, "https://example.com/")))
        }

    @Test
    fun aDeviceWithNoWebViewIsACodeAndNotItsWords() =
        runTest {
            pages.failure = IllegalStateException("No WebView installed; https://secret.example.com/")
            val response = engine.answer(create(4))
            assertRefused("engine.unavailable", response)
            assertEquals("", response.error?.field_)
            assertEquals(0, engine.webViewCount)
        }

    @Test
    fun anAddressIsLoadedInTheWebViewTheCoreNamed() =
        runTest {
            engine.answer(create(4))
            engine.answer(create(5))
            assertDone(engine.answer(load(5, "https://example.com/")))
            assertEquals(emptyList<String>(), pages.made[0].loaded)
            assertEquals(listOf("https://example.com/"), pages.made[1].loaded)
            assertRefused("webview.unknown", engine.answer(load(9, "https://example.com/")))
        }

    @Test
    fun onlyAnAddressTheCoresPolicyAllowsIsEverLoaded() =
        runTest {
            engine.answer(create(4))
            // WebView.loadUrl would run this as script in the page; the core's policy says no.
            core.setVerdict(NavigationVerdict.CANCEL, "javascript:alert(document.cookie)")
            core.setVerdict(NavigationVerdict.EXTERNAL, "mailto:ada@example.com")
            assertRefused("webview.bad_address", engine.answer(load(4, "javascript:alert(document.cookie)")))
            assertRefused("webview.bad_address", engine.answer(load(4, "mailto:ada@example.com")))
            assertRefused("webview.bad_address", engine.answer(load(4, "")))
            assertTrue(pages.made[0].loaded.isEmpty())
        }

    @Test
    fun beforeACoreIsAttachedNothingIsLoaded() =
        runTest {
            val alone = SystemWebViewEngine(pages)
            assertDone(alone.answer(create(4)))
            assertRefused("webview.bad_address", alone.answer(load(4, "https://example.com/")))
            // What a page reports goes nowhere, and what it asks is answered safely.
            val host = pages.made[0].host
            host.report(EngineEvent(web_view_id = 4, navigation_finished = NavigationFinished()))
            assertEquals(NavigationVerdict.CANCEL, host.navigationVerdict("https://example.com/", 4, mainFrame = true, userGesture = true))
            assertEquals(RequestVerdict.ALLOW, host.requestVerdict("https://example.com/a.js", "https://example.com/", 4, ResourceType.OTHER))
        }

    @Test
    fun aWebViewThatCannotLoadSaysTheEngineIsNotThere() =
        runTest {
            engine.answer(create(4))
            pages.made[0].loads = false
            assertRefused("engine.unavailable", engine.answer(load(4, "https://example.com/")))
        }

    @Test
    fun commandsReachTheWebView() =
        runTest {
            engine.answer(create(4))
            val all =
                listOf(
                    NavigationCommand.NAVIGATION_COMMAND_BACK,
                    NavigationCommand.NAVIGATION_COMMAND_FORWARD,
                    NavigationCommand.NAVIGATION_COMMAND_RELOAD,
                    NavigationCommand.NAVIGATION_COMMAND_STOP,
                )
            for (each in all) assertDone(engine.answer(command(4, each)))
            assertEquals(all, pages.made[0].commands)
            assertRefused("webview.bad_command", engine.answer(command(4, NavigationCommand.NAVIGATION_COMMAND_UNSPECIFIED)))
            assertRefused("webview.unknown", engine.answer(command(9, NavigationCommand.NAVIGATION_COMMAND_RELOAD)))
            assertEquals(all, pages.made[0].commands)
        }

    @Test
    fun destroyingReleasesTheWebView() =
        runTest {
            engine.answer(create(4))
            assertDone(engine.answer(destroy(4)))
            assertTrue(pages.made[0].closed)
            assertEquals(0, engine.webViewCount)
            assertNull(engine.webView(4))
            // Also when the shell no longer had it.
            assertDone(engine.answer(destroy(4)))
            assertRefused("webview.unknown", engine.answer(load(4, "https://example.com/")))
            // Its id can be made again: the core never reuses one, but nothing here depends on that.
            assertDone(engine.answer(create(4)))
        }

    // A page that asks for a new window (006)

    private fun windowsAsked(by: Long) = core.reports.filter { it.web_view_id == by }.mapNotNull { it.new_window_requested }

    @Test
    fun aWindowAPageAsksForIsMadeInItsLoginSetNamedByTheShellAndReported() =
        runTest {
            engine.answer(create(4))
            engine.answer(create(5, LoginSet(id = OTHER_LOGIN_SET)))
            val before = engine.generation

            val first = pages.made[0].host.newWindow(4, userGesture = true)
            val second = pages.made[1].host.newWindow(5, userGesture = false)

            assertNotNull(first)
            assertNotNull(second)
            assertEquals(4, engine.webViewCount)
            val made = pages.made.drop(2)
            assertEquals("each in the login set of the page that asked", listOf(LOGIN_SET, OTHER_LOGIN_SET), made.map { it.loginSet })
            assertTrue("named with ids of the shell's own", made.all { it.id and SystemWebViewEngine.OWN_ID != 0L })
            assertTrue("and no id twice", made[0].id != made[1].id)
            // Reported for the web view that asked, with what the WebView said of a gesture.
            assertEquals(listOf(made[0].id to true), windowsAsked(by = 4).map { it.new_web_view_id to it.user_gesture })
            assertEquals(listOf(made[1].id to false), windowsAsked(by = 5).map { it.new_web_view_id to it.user_gesture })
            assertTrue("a window draws again when the web views change", engine.generation > before)
        }

    @Test
    fun theCoresRequestsAboutAWindowAPageAskedForAreAnswered() =
        runTest {
            engine.answer(create(4))
            pages.made[0].host.newWindow(4, userGesture = true)
            val window = pages.made[1]

            assertDone(engine.answer(load(window.id, "https://example.com/")))
            assertEquals(listOf("https://example.com/"), window.loaded)
            assertRefused("webview.exists", engine.answer(create(window.id)))

            // What the core sends for a window it refuses.
            assertDone(engine.answer(destroy(window.id)))
            assertTrue(window.closed)
            assertEquals(1, engine.webViewCount)
            assertFalse("the page that asked is as it was", pages.made[0].closed)

            // The next window has an id of its own; a window's page can ask for one too.
            pages.made[0].host.newWindow(4, userGesture = true)
            assertTrue(pages.made[2].id != window.id)
            pages.made[2].host.newWindow(pages.made[2].id, userGesture = true)
            assertEquals(LOGIN_SET, pages.made[3].loginSet)
            assertEquals(3, engine.webViewCount)
        }

    @Test
    fun noWindowIsMadeForAWebViewThatIsGoneOrWhenNoneCanBeMade() =
        runTest {
            engine.answer(create(4))
            val host = pages.made[0].host

            // Its own web view was destroyed while it asked.
            engine.answer(destroy(4))
            assertNull(host.newWindow(4, userGesture = true))

            engine.answer(create(5))
            // The WebView lost its profiles, or itself, since the page was made.
            pages.keepsLoginSetsApart = false
            assertNull(pages.made[1].host.newWindow(5, userGesture = true))
            pages.keepsLoginSetsApart = true
            pages.failure = IllegalStateException("no WebView")
            assertNull(pages.made[1].host.newWindow(5, userGesture = true))

            assertEquals(1, engine.webViewCount)
            assertEquals(2, pages.made.size)
            assertTrue("nothing was reported", core.reports.none { it.new_window_requested != null })
        }

    @Test
    fun aRequestThisShellDoesNotKnowIsRefusedWithACode() =
        runTest {
            assertRefused("host.unsupported", engine.answer(HostRequest()))
        }

    @Test
    fun aFindReachesThePageOfTheWebViewTheCoreNamed() =
        runTest {
            engine.answer(create(4))
            engine.answer(create(5))
            fun find(id: Long, text: String, backwards: Boolean = false) = HostRequest(web_view_find = WebViewFind(web_view_id = id, text = text, backwards = backwards))

            assertDone(engine.answer(find(5, "needle")))
            assertDone(engine.answer(find(5, "needle", backwards = true)))
            assertDone(engine.answer(find(5, "")))

            assertEquals(listOf("needle" to false, "needle" to true, "" to false), pages.made[1].finds)
            assertTrue("and no other page", pages.made[0].finds.isEmpty())
            assertRefused("webview.unknown", engine.answer(find(9, "needle")))
        }

    @Test
    fun aZoomReachesThePageOfTheWebViewTheCoreNamed() =
        runTest {
            engine.answer(create(4))
            engine.answer(create(5))
            fun zoom(id: Long, percent: Int) = HostRequest(web_view_set_zoom = WebViewSetZoom(web_view_id = id, percent = percent))

            assertDone(engine.answer(zoom(5, 125)))
            assertDone(engine.answer(zoom(5, 100)))

            assertEquals(listOf(125, 100), pages.made[1].zooms)
            assertTrue("and no other page", pages.made[0].zooms.isEmpty())
            assertRefused("webview.unknown", engine.answer(zoom(9, 125)))
        }

    @Test
    fun theEngineItselfOpensNothingOutside() =
        runTest {
            // The System service is the shell's own to answer (007), not a web engine's.
            assertRefused("host.unsupported", engine.answer(HostRequest(open_outside = OpenOutside(url = "mailto:ada@example.com"))))
        }

    @Test
    fun whatAPageReportsAndAsksGoesToTheCore() =
        runTest {
            engine.answer(create(4))
            val host = pages.made[0].host
            host.report(EngineEvent(web_view_id = 4, navigation_finished = NavigationFinished()))
            assertEquals(listOf(4L), core.reports.map { it.web_view_id })

            core.setVerdict(NavigationVerdict.EXTERNAL, "tel:+441632960000")
            core.setVerdict(RequestVerdict.BLOCK, "https://tracker.example.com/pixel.gif")
            assertEquals(NavigationVerdict.EXTERNAL, host.navigationVerdict("tel:+441632960000", 4, mainFrame = true, userGesture = true))
            assertEquals(NavigationVerdict.ALLOW, host.navigationVerdict("https://example.com/", 4, mainFrame = true, userGesture = false))
            assertEquals(RequestVerdict.BLOCK, host.requestVerdict("https://tracker.example.com/pixel.gif", "https://example.com/", 4, ResourceType.OTHER))
        }

    @Test
    fun aLoginSetIsNamedByALowerCaseUuid() {
        assertTrue(SystemWebViewEngine.isUuid(LOGIN_SET))
        assertTrue(SystemWebViewEngine.isUuid("00000000-0000-0000-0000-000000000000"))
        for (bad in listOf("", "Default", LOGIN_SET.uppercase(), LOGIN_SET.replace("-", ""), "$LOGIN_SET ", LOGIN_SET.replace('f', 'g'), "0f8b1c9e-2d47-4a6b-9c3e_5a1d7e8f9a0b", "../../../etc/passwd")) {
            assertFalse(bad, SystemWebViewEngine.isUuid(bad))
        }
    }

    // A private window's login set (037): a profile to throw away

    private fun private(id: Long, loginSet: String = PRIVATE) = create(id, LoginSet(id = loginSet, ephemeral = true))

    @Test
    fun aPrivateLoginSetIsWrittenDownBeforeItsPageIsMadeAndClearedWhenItsLastPageIsGone() =
        runTest {
            assertDone(engine.answer(private(1)))
            // What an earlier run left is looked for first; then the login set is written down
            // as one to throw away; and only then is its page made.
            assertEquals(listOf("sweep", "throw away $PRIVATE", "make $PRIVATE"), pages.did)

            // A second page of it: nothing is cleared while one is left.
            assertDone(engine.answer(private(2)))
            assertDone(engine.answer(destroy(1)))
            assertEquals(emptyList<String>(), pages.did.filter { it.startsWith("clear") })

            // Its last page gone: what its pages stored is removed, once.
            assertDone(engine.answer(destroy(2)))
            assertEquals(listOf("clear $PRIVATE"), pages.did.filter { it.startsWith("clear") })
            assertDone(engine.answer(destroy(2)))
            assertEquals(listOf("clear $PRIVATE"), pages.did.filter { it.startsWith("clear") })
            // What an earlier run left was looked for once, and not again for each page.
            assertEquals(1, pages.did.count { it == "sweep" })
        }

    @Test
    fun aPrivateLoginSetsPagesAreMadeToAskTheKeyboardNotToLearn() =
        runTest {
            assertDone(engine.answer(create(1)))
            assertDone(engine.answer(private(2)))
            assertEquals("the private login set's page, and not the ordinary one's", listOf(false, true), pages.made.map { it.forgetful })

            // A window a page asks for is made in the asking page's login set, and as its
            // pages are: a private page's is a private page.
            assertNotNull(pages.made[1].host.newWindow(2, userGesture = true))
            assertNotNull(pages.made[0].host.newWindow(1, userGesture = true))
            assertEquals(listOf(false, true, true, false), pages.made.map { it.forgetful })

            // And a second tab of each.
            assertDone(engine.answer(private(3)))
            assertDone(engine.answer(create(4)))
            assertEquals(listOf(true, false), pages.made.takeLast(2).map { it.forgetful })
        }

    @Test
    fun anOrdinaryLoginSetIsNeverWrittenDownOrCleared() =
        runTest {
            assertDone(engine.answer(create(1)))
            assertDone(engine.answer(private(2)))
            assertDone(engine.answer(destroy(1)))
            // The ordinary login set's last page is gone, and nothing of it is touched; the
            // private one's page is still there.
            assertEquals(listOf("sweep", "make $LOGIN_SET", "throw away $PRIVATE", "make $PRIVATE"), pages.did)
            // What an earlier run left is looked for before the first page of any kind.
            assertEquals("sweep", pages.did.first())
        }

    @Test
    fun eachPrivateLoginSetIsClearedByItself() =
        runTest {
            val other = "9a0b1c2d-3e4f-4a5b-8c6d-7e8f9a0b1c2d"
            assertDone(engine.answer(private(1)))
            assertDone(engine.answer(private(2, other)))
            assertDone(engine.answer(destroy(1)))
            assertEquals(listOf("clear $PRIVATE"), pages.did.filter { it.startsWith("clear") })
            assertDone(engine.answer(destroy(2)))
            assertEquals(listOf("clear $PRIVATE", "clear $other"), pages.did.filter { it.startsWith("clear") })
        }

    private fun saveState(id: Long) = HostRequest(web_view_save_state = WebViewSaveState(web_view_id = id))

    private fun restore(id: Long, state: String, url: String) = HostRequest(web_view_restore = WebViewRestore(web_view_id = id, state = state.encodeUtf8(), url = url))

    @Test
    fun aWebViewsStateIsWhatItsPageGivesAndNoneWhenItGivesNone() =
        runTest {
            engine.answer(create(4))
            // A page with none to give: answered, with none.
            val none = engine.answer(saveState(4))
            assertNull(none.error)
            assertEquals(0, checkNotNull(none.web_view_state).state.size)
            pages.made[0].hasState = "the page's own".toByteArray()
            assertEquals("the page's own", engine.answer(saveState(4)).web_view_state?.state?.utf8())
            assertEquals(2, pages.made[0].statesAsked)
            assertRefused("webview.unknown", engine.answer(saveState(9)))
        }

    @Test
    fun aWebViewIsMadeFromAStateWithTheAddressItCameWith() =
        runTest {
            engine.answer(create(4))
            assertDone(engine.answer(restore(4, "a state", "https://example.com/")))
            assertEquals(listOf("a state" to "https://example.com/"), pages.made[0].restored)
            assertTrue("nothing is loaded beside it", pages.made[0].loaded.isEmpty())
            // A page that does not take the state loads the address.
            engine.answer(create(5))
            pages.made[1].takesStates = false
            assertDone(engine.answer(restore(5, "a state", "https://example.com/")))
            assertEquals(listOf("https://example.com/"), pages.made[1].loaded)
            assertRefused("webview.unknown", engine.answer(restore(9, "a state", "https://example.com/")))
        }

    @Test
    fun aStateIsFollowedOnlyToAnAddressTheCoresPolicyAllows() =
        runTest {
            engine.answer(create(4))
            core.setVerdict(NavigationVerdict.CANCEL, "https://refused.example.com/")
            assertRefused("webview.bad_address", engine.answer(restore(4, "a state", "https://refused.example.com/")))
            assertRefused("webview.bad_address", engine.answer(restore(4, "a state", "")))
            assertTrue(pages.made[0].restored.isEmpty())
            assertTrue(pages.made[0].loaded.isEmpty())
            // Before a core is attached, nothing is restored either.
            val alone = SystemWebViewEngine(pages)
            assertDone(alone.answer(create(6)))
            assertRefused("webview.bad_address", alone.answer(restore(6, "a state", "https://example.com/")))
            // And a web view that cannot be made to go anywhere says the engine is not there.
            pages.made[0].loads = false
            assertRefused("engine.unavailable", engine.answer(restore(4, "a state", "https://example.com/")))
        }
}

private const val PRIVATE = "1a2b3c4d-5e6f-4a7b-8c9d-0e1f2a3b4c5d"
