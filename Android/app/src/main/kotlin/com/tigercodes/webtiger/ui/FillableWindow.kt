// What the window's views are in, so that the system's Autofill reaches a page's form (032).
package com.tigercodes.webtiger.ui

import android.content.Context
import android.view.View
import android.view.ViewGroup
import android.view.ViewStructure
import android.webkit.WebView
import android.widget.FrameLayout

/**
 * The window's own view: what Compose draws the window in is in it, and nothing else.
 *
 * The system finds what a password manager can fill by asking the window's views to describe
 * themselves, each asking the views in it. The view Compose draws in describes the window's own
 * text fields and, in the Compose this app is built with (1.11), stops there: the web view,
 * which is inside it, is never asked, and a password manager shown a page's sign-in form sees
 * the address field and nothing of the page (seen on the emulator). So this view, asked to
 * describe itself, also has the web view that is on the screen describe itself, beside what
 * Compose described.
 *
 * Compose 1.12 asks the views inside it itself. Built with that, this would describe the web
 * view twice: the app's test of a sign-in form (AppAutofillTest) says so, and this goes then.
 */
internal class FillableWindow(context: Context) : FrameLayout(context) {
    init {
        // Asked to describe itself, and not passed over for what is in it: a view that says
        // nothing of whether it is to be filled is one the system walks past.
        importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_YES
    }

    override fun dispatchProvideAutofillStructure(structure: ViewStructure, flags: Int) {
        super.dispatchProvideAutofillStructure(structure, flags)
        val pages = ArrayList<WebView>()
        webViewsIn(this, pages)
        if (pages.isEmpty()) return
        var next = structure.addChildCount(pages.size)
        for (page in pages) {
            page.dispatchProvideAutofillStructure(structure.newChild(next), flags)
            next += 1
        }
    }

    /** The web views that are in the window now: the one on the screen. */
    private fun webViewsIn(view: View, found: MutableList<WebView>) {
        if (view is WebView) {
            found += view
        } else if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewsIn(view.getChildAt(index), found)
        }
    }
}
