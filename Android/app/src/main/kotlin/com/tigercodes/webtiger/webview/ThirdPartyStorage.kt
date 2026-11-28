// A third party's storage (071). The System WebView hands a site in a frame of another site's
// page the storage that site has as itself: a site that is a piece of many pages can keep a
// note there and know a person by it wherever it is a piece. The WebView gives an app no
// setting for it. The adapter puts a script in every frame, before any of the page's own: a
// frame the core says is to be kept apart is given a store that goes with its document, and
// is refused the stores that last.
//
// It is not a wall. What a script takes away, a script that looks for another door may find:
// a worker the frame starts has databases of its own, which no script here reaches. It stops
// the plain use of storage to follow a person.
package com.tigercodes.webtiger.webview

import android.annotation.SuppressLint
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature

internal object ThirdPartyStorage {
    /** The name the question has in a frame, for as long as it takes the script to take it away. */
    const val NAME = "webtigerStorage"

    /** The longest origin that is asked about. A longer one is nobody's, and is kept apart. */
    const val LONGEST_ORIGIN = 2048

    /**
     * The question a frame's script asks as its document starts, and waits for: is this frame,
     * in this page, to be kept from its site's own storage? Called on a thread of the
     * WebView's own, so nothing of the web view is touched. What arrives is a page's to say
     * for the moment before the script has taken the name away: it is taken for text and
     * nothing else, and what is not text, or is too long to be an origin, is kept apart.
     */
    class Question(private val keptApart: (origin: String, topOrigin: String) -> Boolean) {
        @JavascriptInterface
        fun apart(origin: String?, topOrigin: String?): Boolean {
            if (origin == null || topOrigin == null) return true
            if (origin.length > LONGEST_ORIGIN || topOrigin.length > LONGEST_ORIGIN) return true
            return try {
                keptApart(origin, topOrigin)
            } catch (_: RuntimeException) {
                true
            }
        }
    }

    /**
     * Puts the script, and its question, in every frame of a web view, where the WebView can
     * be given a script that runs before a page's own. One that cannot is left as it is: a
     * third party has its storage there.
     */
    @SuppressLint("JavascriptInterface", "AddJavascriptInterface") // One method, which answers yes or no; the script takes the name out of the page.
    fun keepApart(view: WebView, apart: (origin: String, topOrigin: String) -> Boolean) {
        if (!WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) return
        view.addJavascriptInterface(Question(apart), NAME)
        WebViewCompat.addDocumentStartJavaScript(view, SCRIPT, setOf("*"))
    }

    /**
     * The script, put in every page and every frame as its document starts. It takes the
     * question for itself and takes its name out of the page, before any script of the page's
     * runs. The page itself, and a frame of the page's own origin, ask nothing and are left
     * as they are. Any other frame asks, with its own origin and the origin of the top frame
     * above it, and is kept apart unless it is told otherwise: when there is nobody to ask,
     * or the asking fails, it is apart.
     *
     * Apart: the page's storage and the session's are each a store of this document's own,
     * empty at first and gone with it; a database, a cache and the private file system are
     * refused, as an engine refuses them where it denies them; and a worker that serves
     * pages, and a shared one, are not there to be found.
     */
    val SCRIPT =
        """
        (function () {
            'use strict';
            var ask = window.$NAME;
            try { delete window.$NAME; } catch (ignored) {}
            if (window.top === window) { return; }
            var own = '', top = '';
            try { own = String(self.origin); } catch (ignored) {}
            try {
                var above = location.ancestorOrigins;
                if (above && above.length) { top = String(above[above.length - 1]); }
            } catch (ignored) {}
            if (own !== '' && own !== 'null' && own === top) { return; }
            var apart = true;
            try { apart = ask.apart(own, top) !== false; } catch (ignored) {}
            if (!apart) { return; }

            function refusal() { return new DOMException('The operation is insecure.', 'SecurityError'); }
            function refuse() { throw refusal(); }
            function refused() { return Promise.reject(refusal()); }
            function fix(on, name, value) {
                try {
                    Object.defineProperty(on, name, { get: function () { return value; }, set: undefined, enumerable: true, configurable: false });
                } catch (ignored) {}
            }
            function has(on, name) { return Object.prototype.hasOwnProperty.call(on, name); }

            // A store that works as the page's storage does, and is this document's alone.
            function store() {
                var kept = Object.create(null);
                var does = {
                    getItem: function (name) { name = String(name); return name in kept ? kept[name] : null; },
                    setItem: function (name, value) { kept[String(name)] = String(value); },
                    removeItem: function (name) { delete kept[String(name)]; },
                    clear: function () { kept = Object.create(null); },
                    key: function (index) { var names = Object.keys(kept); index = index >>> 0; return index < names.length ? names[index] : null; }
                };
                var like = typeof Storage === 'function' ? Object.create(Storage.prototype) : {};
                return new Proxy(like, {
                    get: function (target, name) {
                        if (typeof name !== 'string') { return Reflect.get(target, name); }
                        if (name === 'length') { return Object.keys(kept).length; }
                        if (has(does, name)) { return does[name]; }
                        return name in kept ? kept[name] : undefined;
                    },
                    set: function (target, name, value) {
                        if (typeof name === 'string' && name !== 'length' && !has(does, name)) { kept[name] = String(value); }
                        return true;
                    },
                    has: function (target, name) { return typeof name === 'string' && (name === 'length' || has(does, name) || name in kept); },
                    deleteProperty: function (target, name) { if (typeof name === 'string') { delete kept[name]; } return true; },
                    ownKeys: function () { return Object.keys(kept); },
                    getOwnPropertyDescriptor: function (target, name) {
                        if (typeof name !== 'string' || !(name in kept)) { return undefined; }
                        return { value: kept[name], writable: true, enumerable: true, configurable: true };
                    },
                    defineProperty: function (target, name, described) {
                        if (typeof name === 'string' && described && 'value' in described) { kept[name] = String(described.value); }
                        return true;
                    }
                });
            }
            fix(window, 'localStorage', store());
            fix(window, 'sessionStorage', store());

            // The stores that last: refused.
            fix(window, 'indexedDB', { open: refuse, deleteDatabase: refuse, cmp: refuse, databases: refused });
            if ('openDatabase' in window) { fix(window, 'openDatabase', refuse); }
            if ('caches' in window) { fix(window, 'caches', { open: refused, has: refused, delete: refused, keys: refused, match: refused }); }
            try {
                if (typeof StorageManager === 'function' && StorageManager.prototype.getDirectory) {
                    Object.defineProperty(StorageManager.prototype, 'getDirectory', { value: function () { return refused(); }, writable: false, enumerable: true, configurable: false });
                }
            } catch (ignored) {}

            // The workers that outlive a page: not there.
            try { delete Navigator.prototype.serviceWorker; } catch (ignored) {}
            if ('serviceWorker' in navigator) { fix(navigator, 'serviceWorker', undefined); }
            try { delete window.SharedWorker; } catch (ignored) {}
            if ('SharedWorker' in window) { fix(window, 'SharedWorker', undefined); }
        }());
        """.trimIndent()
}
