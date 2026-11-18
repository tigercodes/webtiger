// Isolation (000 §8.4): a security property, tested as one, on the device's real WebView. Sign
// in under one login set; another must see no cookie and no storage, and deleting a login set
// must leave nothing. A login set is a WebView profile (003 §5.2).
package com.tigercodes.webtiger.webview

import android.content.Context
import android.util.Log
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

    @Test
    fun aPrivateLoginSetIsRefusedNotApproximated() {
        // A WebView profile is always written to disk; the contract's private login set is kept
        // in memory. So there is none to give (003 §5.2, A4).
        assertEquals("engine.no_private", create(1, ephemeral = true).error?.code)
        assertNull(webView(1))
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
        assertTrue("the login set had cookies, and they were removed", cookiesCleared.get(10, TimeUnit.SECONDS))
        assertTrue(dataCleared.get(10, TimeUnit.SECONDS))

        create(2, loginSet = loginSet)
        load(2, site.url("/whoami"))
        await(2, "the cookie's absence") { finished(it) && lastTitle(2).startsWith("who=") }
        assertEquals("who=none", lastTitle(2))
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
    }
}
