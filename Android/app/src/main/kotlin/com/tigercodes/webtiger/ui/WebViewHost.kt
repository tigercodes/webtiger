// Shows the web view the core named. The view is the engine's; this only gives it a place.
package com.tigercodes.webtiger.ui

import android.content.ContextWrapper
import android.content.MutableContextWrapper
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import com.tigercodes.webtiger.webview.SystemWebViewEngine

/**
 * A place for the web view `webViewId`. The web view lives in the engine, not in the
 * composition: leaving this place, or the activity being made again, reloads nothing.
 */
@Composable
internal fun WebViewHost(engine: SystemWebViewEngine, webViewId: Long, modifier: Modifier = Modifier) {
    AndroidView(
        modifier = modifier,
        factory = { context -> FrameLayout(context) },
        update = { place ->
            // Read so that this runs again when the engine's web views change: the one named
            // may arrive after the snapshot that names it, or be replaced.
            engine.generation
            place.show(engine.webView(webViewId))
        },
        onRelease = { place -> place.show(null) },
    )
}

/** Puts `view` in this place, and takes out whatever was there. */
private fun FrameLayout.show(view: View?) {
    val shown = getChildAt(0)
    if (shown === view) return
    if (shown != null) {
        removeView(shown)
        // A web view keeps no hold on an activity it is no longer in.
        shown.lent()?.baseContext = context.applicationContext
    }
    if (view == null) return
    (view.parent as? ViewGroup)?.removeView(view)
    // While it is on screen the web view borrows the activity: its menus and dialogs need one.
    view.lent()?.baseContext = context
    addView(view, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
}

/**
 * The part of a web view's context that an activity is lent to. It is not the context the web
 * view was made with, but inside it: the adapter wraps that part in a context of its own, which
 * hears what the WebView asks to have started (webview/SearchingContext.kt). Looked for by
 * what it is, however deep: looked for only at the top, it was not found once it was wrapped,
 * and no web view was lent an activity at all (032).
 */
internal fun View.lent(): MutableContextWrapper? {
    var inside = context
    while (inside is ContextWrapper) {
        if (inside is MutableContextWrapper) return inside
        inside = inside.baseContext
    }
    return null
}
