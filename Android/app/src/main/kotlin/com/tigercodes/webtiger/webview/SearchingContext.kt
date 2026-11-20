// Searching for words that are selected (023), on the System WebView.
//
// The WebView's bar for selected text has an item of its own that searches the web for the
// words. What it does is ask the system to start whatever answers a web search: another app,
// with a search engine of its own choosing. A person who chose where their searches go (020)
// would have them go elsewhere, and leave the browser to do it.
//
// The WebView offers that item only when it can see something that answers a web search. An
// app sees only what it has asked to see, and its own parts: so the manifest names an activity
// of the app's own as such an answerer. It is not exported: no other app sees it or can start
// it. With it there, the WebView offers its item; without it, it offered none (WebView 133).
//
// The WebView starts the search by asking its context to. The WebViews the adapter makes are
// made with this context, which takes that one request for what it is, the person asking to
// search for the words, and tells the adapter: nothing is started. Every other request goes on
// as it was. The WebView decides when its item is shown (not for what is typed in a password's
// field), and it is the WebView that names the words.
package com.tigercodes.webtiger.webview

import android.app.Activity
import android.app.SearchManager
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.os.Bundle

internal class SearchingContext(base: Context, private val search: (String) -> Unit) : ContextWrapper(base) {
    override fun startActivity(intent: Intent) = startActivity(intent, null)

    override fun startActivity(intent: Intent, options: Bundle?) {
        if (intent.action == Intent.ACTION_WEB_SEARCH) {
            search(intent.getStringExtra(SearchManager.QUERY).orEmpty())
        } else {
            super.startActivity(intent, options)
        }
    }
}

/**
 * What the manifest names so that the WebView offers its search for selected words: something
 * of the app's own that answers a web search. It is never shown: the adapter hears the
 * WebView's request before anything is started ([SearchingContext]). Started some other way,
 * it does nothing and goes.
 */
class SearchSelectionActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        finish()
    }
}
