// Isolation (000 §8.4): a security property, tested as one, on the device's real WebView. Sign
// in under one login set; another must see no cookie and no storage, and deleting a login set
// must leave nothing. A login set is a WebView profile (003 §5.2).
package com.tigercodes.webtiger.webview

import android.content.Context
import android.util.Log
import android.view.inputmethod.EditorInfo
import android.webkit.CookieManager
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.webkit.ProfileStore
import androidx.webkit.WebStorageCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewIsolationTest : AdapterTestCase() {
    @Test
    fun thisDevicesWebViewKeepsLoginSetsApart() {
        // Everything below rests on it. Without profiles the engine makes no web view at all
        // (`engine.no_profiles`), and these tests would say so rather than pass.
        assertTrue("the device's WebView has profiles", onMain { WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE) })
    }

    @Test
    fun loginSetsShareNoCookies() {
        val personal = newLoginSet()
        val work = newLoginSet()
        create(1, loginSet = personal)
        create(2, loginSet = work)
        create(3, loginSet = personal)

        // Sign in under the first login set.
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }

        // Another login set sees nothing.
        load(2, site.url("/whoami"))
        await(2, "the other login set's answer") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("a Space with its own logins must not see another's cookie", "who=none", lastTitle(2))

        // A second web view in the first login set is signed in.
        load(3, site.url("/whoami"))
        await(3, "the same login set's answer") { finished(it) && lastTitle(3).startsWith("who=") }
        assertEquals("Spaces that share logins share them", "who=ada", lastTitle(3))
    }

    @Test
    fun loginSetsShareNoLocalStorage() {
        create(1, loginSet = newLoginSet())
        create(2, loginSet = newLoginSet())
        load(1, site.url("/storage?v=ada"))
        await(1, "the value to be stored") { lastTitle(1) == "stored=ada" }
        load(2, site.url("/storage"))
        await(2, "the other login set's answer") { lastTitle(2).startsWith("stored=") }
        assertEquals("stored=none", lastTitle(2))
    }

    @Test
    fun loginSetsShareNoDatabasesAndNoCaches() {
        val personal = newLoginSet()
        create(1, loginSet = personal)
        create(2, loginSet = newLoginSet())
        create(3, loginSet = personal)
        for (id in listOf(1L, 2L, 3L)) {
            load(id, site.url("/"))
            await(id, "the page") { finished(it) }
        }
        // Under the first login set: a database with a record in it, and a cache with an entry.
        val stored =
            run(
                1,
                """
                await new Promise((resolve, reject) => {
                    const opening = indexedDB.open("who", 1);
                    opening.onupgradeneeded = () => opening.result.createObjectStore("names");
                    opening.onerror = () => reject(opening.error);
                    opening.onsuccess = () => {
                        const writing = opening.result.transaction("names", "readwrite");
                        writing.objectStore("names").put("ada", "who");
                        writing.oncomplete = () => resolve();
                        writing.onerror = () => reject(writing.error);
                    };
                });
                const cache = await caches.open("pages");
                await cache.put("/kept", new Response("ada"));
                return true;
                """.trimIndent(),
            )
        assertEquals("the page could use IndexedDB and the Cache API", "true", stored)

        val count = "return [(await indexedDB.databases()).length, (await caches.keys()).length];"
        assertEquals("another login set sees no database and no cache", "[0,0]", run(2, count))
        assertEquals("the same login set sees both", "[1,1]", run(3, count))
    }

    @Test
    fun loginSetsShareNoServiceWorkers() {
        val personal = newLoginSet()
        create(1, loginSet = personal)
        create(2, loginSet = newLoginSet())
        create(3, loginSet = personal)
        for (id in listOf(1L, 2L, 3L)) {
            load(id, site.url("/"))
            await(id, "the page") { finished(it) }
        }
        assertEquals("the WebView offers a page on the loopback address service workers", "true", run(1, "return 'serviceWorker' in navigator;"))
        val registered = run(1, "const registration = await navigator.serviceWorker.register('/worker.js'); return registration.scope;")
        assertEquals("\"${site.url("/")}\"", registered)
        val count = "return (await navigator.serviceWorker.getRegistrations()).length;"
        assertEquals("another login set has no worker for the same site", "0", run(2, count))
        assertEquals("the same login set has it", "1", run(3, count))
    }

    /** How many times the site was asked for a path. */
    private fun asked(path: String): Int = site.requests.count { it == path }

    @Test
    fun loginSetsShareNoHttpCache() {
        val personal = newLoginSet()
        create(1, loginSet = personal)
        create(2, loginSet = newLoginSet())

        // A page whose script may be kept for an hour.
        load(1, site.url("/uses-cached"))
        await(1, "the script to run") { lastTitle(1) == "Cached" }
        assertEquals(1, asked("/cached.js"))

        // Another web view in the same login set gets the script from the cache. The cache is
        // written a moment after the answer came, so this is tried until it holds; without it
        // the next step would show nothing.
        var kept = false
        for (attempt in 0 until 20) {
            val before = asked("/cached.js")
            val id = 100L + attempt
            create(id, loginSet = personal)
            load(id, site.url("/uses-cached"))
            await(id, "the script to run again") { lastTitle(id) == "Cached" }
            kept = asked("/cached.js") == before
            destroy(id)
            if (kept) break
            Thread.sleep(200)
        }
        assertTrue("web views in one login set share its cache", kept)

        // Another login set has no copy: the site is asked again.
        val before = asked("/cached.js")
        load(2, site.url("/uses-cached"))
        await(2, "the other login set's script") { lastTitle(2) == "Cached" }
        assertEquals("a login set's cache is its own", before + 1, asked("/cached.js"))
    }

    @Test
    fun aSignInForTheSessionOutlivesTheLastWebViewOfItsLoginSet() {
        val loginSet = newLoginSet()
        create(1, loginSet = loginSet)
        // A cookie with no expiry, which an engine keeps in memory.
        load(1, site.url("/session?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Session")(it) }
        // The login set's only tab sleeps: it has no web view at all for a while.
        destroy(1)
        Thread.sleep(2_000)
        create(2, loginSet = loginSet)
        load(2, site.url("/whoami"))
        await(2, "the woken tab's answer") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("a tab that sleeps does not sign its Space out", "who=ada", lastTitle(2))
    }

    // A private window's login set (037): a profile to throw away

    /** What a login set's own jar holds for the test site. */
    private fun jar(loginSet: String): String = onMain { ProfileStore.getInstance().getProfile(loginSet)?.cookieManager?.getCookie(site.url("/whoami")) }.orEmpty()

    /**
     * The WebView has no login set in memory: a private window's is a profile like any other,
     * with its own cookies and storage while its pages are there; with its last page gone,
     * what they stored is removed, and a page made in it afterwards finds nothing.
     */
    @Test
    fun aPrivateLoginSetIsClearedWhenItsLastPageIsGone() {
        val loginSet = newLoginSet()
        assertNull(create(1, loginSet = loginSet, ephemeral = true).error)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/storage?v=ada"))
        await(1, "the value to be stored") { lastTitle(1) == "stored=ada" }
        // A second page of it, and the first one gone: nothing is removed while a page is left.
        assertNull(create(2, loginSet = loginSet, ephemeral = true).error)
        destroy(1)
        load(2, site.url("/whoami"))
        await(2, "the cookie, still there") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("who=ada", lastTitle(2))
        assertTrue("in the jar: <${jar(loginSet)}>", jar(loginSet).contains("ada"))

        destroy(2)

        // Its last page gone: the jar empties (the WebView does it a moment later).
        var left = jar(loginSet)
        for (attempt in 0 until 100) {
            if (left.isEmpty()) break
            Thread.sleep(100)
            left = jar(loginSet)
        }
        assertEquals("the private login set's jar is empty once its last page is gone", "", left)
        // And a page made in it afterwards finds no cookie and no stored value.
        assertNull(create(3, loginSet = loginSet, ephemeral = true).error)
        load(3, site.url("/whoami"))
        await(3, "the cookie's absence") { finished(it) && lastTitle(3).startsWith("who=") }
        assertEquals("who=none", lastTitle(3))
        var stored = ""
        for (attempt in 0 until 50) {
            stored = run(3, "return [localStorage.getItem('who'), (await indexedDB.databases()).length]").orEmpty()
            if (stored == "[null,0]") break
            Thread.sleep(100)
        }
        assertEquals("no stored value and no database is left", "[null,0]", stored)
    }

    /** A private login set and an ordinary one share nothing, either way: it is a login set like any other while it lives. */
    @Test
    fun aPrivateLoginSetSharesNothingWithAnOrdinaryOne() {
        val (ordinary, private) = newLoginSet() to newLoginSet()
        create(1, loginSet = ordinary)
        load(1, site.url("/set?v=ada"))
        await(1, "the ordinary cookie") { finished(it) && hasTitle("Set")(it) }
        assertNull(create(2, loginSet = private, ephemeral = true).error)
        load(2, site.url("/whoami"))
        await(2, "who the private page is") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("the ordinary login set's cookie is not the private one's", "who=none", lastTitle(2))

        load(2, site.url("/set?v=grace"))
        await(2, "the private cookie") { reports -> reports.count { it.title_changed?.title == "Set" } >= 1 && finished(reports, times = 2) }
        load(1, site.url("/whoami"))
        await(1, "who the ordinary page is") { finished(it, times = 2) && lastTitle(1).startsWith("who=") }
        assertEquals("the private login set's cookie is not the ordinary one's", "who=ada", lastTitle(1))
    }

    /**
     * A private login set's web view asks the keyboard not to learn from what is typed in a
     * page's field (037 §5.5); an ordinary one's asks nothing of the kind. Each is asked as the
     * system asks a view about the field that is being typed in, once a finger has put the
     * cursor in the page's field.
     */
    @Test
    fun aPrivateLoginSetsWebViewAsksTheKeyboardNotToLearn() {
        create(1)
        assertNull(create(2, ephemeral = true).error)
        val told = HashMap<Long, Int>()
        for (id in listOf(1L, 2L)) {
            load(id, site.url("/sign-in-form"))
            await(id, "the form") { finished(it) }
            tap(id, "user")
            val view = checkNotNull(webView(id))
            // The web view has a field for the keyboard a moment after the finger: until
            // then it has nothing to say, and says nothing.
            var options: Int? = null
            for (attempt in 0 until 100) {
                options = onMain { EditorInfo().let { info -> view.onCreateInputConnection(info)?.let { info.imeOptions } } }
                if (options != null) break
                Thread.sleep(100)
            }
            told[id] = checkNotNull(options) { "web view $id never had a field for the keyboard" }
        }
        assertEquals("an ordinary login set's page asks nothing of the kind", 0, told.getValue(1) and EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING)
        assertTrue("a private login set's page asks the keyboard not to learn", told.getValue(2) and EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING != 0)
    }

    /**
     * What an earlier run's private windows left is deleted before anything is made in this
     * run: the only time the WebView lets a profile be deleted (§5.2). The first web view made
     * in this process, by whichever test, did it; this looks at what it did. On a device's
     * first run there is nothing left to delete: this run leaves one for the next.
     */
    @Test
    fun whatAnEarlierRunsPrivateWindowsLeftIsDeletedBeforeAnythingIsMade() {
        assertNull(create(1, loginSet = newLoginSet(), ephemeral = true).error)
        val left = checkNotNull(PrivateProfiles.leftAtStart) { "the list was looked at before the first web view was made" }
        val deleted = PrivateProfiles.deletedAtStart
        val there = onMain { ProfileStore.getInstance().allProfileNames }
        Log.i("WTFACT", "private login sets left by an earlier run: ${left.size}; their profiles deleted as this run started: ${deleted.size}")
        assertEquals("every one that was left was deleted: none can be in use as a run starts", left.toSet(), deleted.toSet())
        for (name in deleted) assertFalse("a deleted profile is no longer among the WebView's", name in there)
    }

    @Test
    fun theDefaultProfileIsNeverUsed() {
        create(1, loginSet = newLoginSet())
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        // The default profile is what a web view without a profile of its own would use. No
        // login set's cookie may be found there.
        val leaked = onMain { CookieManager.getInstance().getCookie(site.url("/")) }
        assertNull("nothing a login set stores reaches the default profile", leaked)
    }

    /**
     * Not an assertion about WebTiger: a record of what the System WebView offers a page
     * (000 §4.7, spike Q11), written to the log for 001.
     */
    @Test
    fun whatTheSystemWebViewOffersAPage() {
        create(1, loginSet = newLoginSet())
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        val offered =
            run(
                1,
                """
                const keySystem = async (name, type) => {
                    if (!navigator.requestMediaKeySystemAccess) return false;
                    try {
                        await navigator.requestMediaKeySystemAccess(name, [{
                            initDataTypes: [type], videoCapabilities: [{ contentType: 'video/mp4; codecs="avc1.42E01E"' }],
                        }]);
                        return true;
                    } catch (refused) {
                        return false;
                    }
                };
                const offered = {
                    secureContext: window.isSecureContext,
                    serviceWorker: "serviceWorker" in navigator,
                    push: "PushManager" in window,
                    notifications: "Notification" in window,
                    credentials: "credentials" in navigator,
                    usb: "usb" in navigator,
                    serial: "serial" in navigator,
                    hid: "hid" in navigator,
                    bluetooth: "bluetooth" in navigator,
                    storageEstimate: !!(navigator.storage && navigator.storage.estimate),
                    widevine: await keySystem("com.widevine.alpha", "cenc"),
                    clearKey: await keySystem("org.w3.clearkey", "keyids"),
                };
                return Object.keys(offered).sort().map(name => name + "=" + offered[name]).join(" ");
                """.trimIndent(),
            )
        assertNotNull("the page answered", offered)
        Log.i("WTFACT", "System WebView offers a page: $offered")
    }

    /** Whether the WebView refuses, now, to delete a login set's profile. */
    private fun deletionIsRefused(loginSet: String): Boolean =
        onMain {
            try {
                ProfileStore.getInstance().deleteProfile(loginSet)
                false
            } catch (_: IllegalStateException) {
                true
            }
        }

    /**
     * Not WebTiger's behaviour: the platform's, pinned here because W2's "delete this Space's
     * logins" rests on it. A WebView profile that a web view has used cannot be deleted while
     * the app runs, not even once the web view is gone. So on Android a login set is cleared at
     * once (the next test) and its profile deleted when the app next starts.
     */
    @Test
    fun aLoginSetUsedInThisRunCannotBeDeletedBeforeTheAppStartsAgain() {
        val loginSet = newLoginSet()
        create(1, loginSet = loginSet)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        assertTrue("the login set's profile exists", onMain { ProfileStore.getInstance().allProfileNames.contains(loginSet) })
        assertTrue("a login set a web view uses is not deleted under it", deletionIsRefused(loginSet))

        destroy(1)
        // A destroyed web view lets go a moment later on other engines. Not here: wait as long
        // as the Apple suite ever does, and the answer is the same.
        Thread.sleep(2_000)
        assertTrue("the WebView keeps a profile it has loaded until the process ends", deletionIsRefused(loginSet))
        assertTrue(onMain { ProfileStore.getInstance().allProfileNames.contains(loginSet) })

        // Asking for a profile is enough to hold it: no web view ever used this one.
        val unused = newLoginSet()
        onMain { ProfileStore.getInstance().getOrCreateProfile(unused) }
        assertTrue("even a profile only asked for is held", deletionIsRefused(unused))
    }

    @Test
    fun clearingALoginSetLeavesNothing() {
        val loginSet = newLoginSet()
        create(1, loginSet = loginSet)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/storage?v=ada"))
        await(1, "the value to be stored") { lastTitle(1) == "stored=ada" }
        val stored =
            run(
                1,
                """
                await new Promise((resolve, reject) => {
                    const opening = indexedDB.open("who", 1);
                    opening.onupgradeneeded = () => opening.result.createObjectStore("names");
                    opening.onerror = () => reject(opening.error);
                    opening.onsuccess = () => { opening.result.close(); resolve(); };
                });
                const cache = await caches.open("pages");
                await cache.put("/kept", new Response("ada"));
                await navigator.serviceWorker.register('/worker.js');
                return true;
                """.trimIndent(),
            )
        assertEquals("the page stored a database, a cache and a service worker", "true", stored)
        destroy(1)

        // What the platform gives for clearing a profile that stays. A current WebView clears
        // everything a site stored in one call; an older one clears cookies and storage only.
        val thorough = onMain { WebViewFeature.isFeatureSupported(WebViewFeature.DELETE_BROWSING_DATA) }
        val cookiesCleared = CompletableFuture<Boolean>()
        val dataCleared = CompletableFuture<Boolean>()
        // The premise, looked at in the login set's own jar: the cookie is there to be cleared.
        // (The system's word for whether it removed any is not it: see the end.)
        fun inTheJar(): String = onMain { checkNotNull(ProfileStore.getInstance().getProfile(loginSet)).cookieManager.getCookie(site.url("/whoami")) }.orEmpty()
        val before = inTheJar()
        assertTrue("the login set has the cookie before it is cleared; its jar says <$before>", before.contains("ada"))
        onMain {
            val profile = checkNotNull(ProfileStore.getInstance().getProfile(loginSet))
            profile.cookieManager.removeAllCookies { removed -> cookiesCleared.complete(removed) }
            if (thorough) {
                WebStorageCompat.deleteBrowsingData(profile.webStorage) { dataCleared.complete(true) }
            } else {
                profile.webStorage.deleteAllData()
                dataCleared.complete(true)
            }
        }
        // What the system says of whether it removed any cookie is not relied on: on the first
        // run after the device starts it now and then says it removed none, with the cookie
        // there before and gone after (003 §8.1). What is in the jar is.
        val saidRemoved = cookiesCleared.get(10, TimeUnit.SECONDS)
        assertTrue(dataCleared.get(10, TimeUnit.SECONDS))
        assertEquals("the login set's jar is empty once it is cleared", "", inTheJar())

        create(2, loginSet = loginSet)
        load(2, site.url("/whoami"))
        await(2, "the cookie's absence") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("no cookie is left (the system said it removed some: $saidRemoved)", "who=none", lastTitle(2))
        // The older call says nothing when it is done: its storage goes a moment later.
        var left = ""
        for (attempt in 0 until 50) {
            left = run(2, "return [localStorage.getItem('who'), (await indexedDB.databases()).length, (await caches.keys()).length, (await navigator.serviceWorker.getRegistrations()).length];").orEmpty()
            if (left.startsWith("[null,0,")) break
            Thread.sleep(100)
        }
        Log.i("WTFACT", "WebView ${onMain { WebViewCompat.getCurrentWebViewPackage(ApplicationProvider.getApplicationContext<Context>())?.versionName }}: DELETE_BROWSING_DATA=$thorough; left after clearing [localStorage, databases, caches, service workers] = $left")
        if (thorough) {
            assertEquals("no stored value, no database, no cache and no service worker is left", "[null,0,0,0]", left)
        } else {
            // What WebTiger must not promise on such a WebView: the Cache API's entries and the
            // service workers outlive `deleteAllData`. They go with the profile, at the next start.
            assertEquals("an older WebView clears the stored value and the database, and leaves the cache and the worker", "[null,0,1,1]", left)
        }
        Log.i("WTFACT", "removeAllCookies said it removed some: $saidRemoved; the jar had <$before> before and nothing after")
    }
}
