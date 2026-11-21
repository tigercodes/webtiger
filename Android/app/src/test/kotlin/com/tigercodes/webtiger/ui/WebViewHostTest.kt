// The part of a web view's context that the window lends its activity to (032): found however
// the adapter has wrapped it. On the JVM, with plain views: what is tested is the looking.
package com.tigercodes.webtiger.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.MutableContextWrapper
import android.view.View
import androidx.test.core.app.ApplicationProvider
import com.tigercodes.webtiger.webview.SearchingContext
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class WebViewHostTest {
    private val application: Context = ApplicationProvider.getApplicationContext()

    @Test
    fun whatIsLentAnActivityIsFoundInsideWhateverItIsWrappedIn() {
        val lendable = MutableContextWrapper(application)

        // By itself, as web views were made before the adapter wrapped their context.
        assertSame(lendable, View(lendable).lent())
        // Inside the adapter's own context, which hears what the WebView asks to have started
        // (023): the way every web view is made. Looked for only at the top, it was not found,
        // and no web view was lent an activity.
        assertSame(lendable, View(SearchingContext(lendable) {}).lent())
        // And however deep.
        assertSame(lendable, View(ContextWrapper(ContextWrapper(SearchingContext(lendable) {}))).lent())
        // The outermost of two is the one a web view's own context is.
        val outer = MutableContextWrapper(ContextWrapper(lendable))
        assertSame(outer, View(outer).lent())
    }

    @Test
    fun aViewWhoseContextCanBeLentNothingHasNone() {
        assertNull(View(application).lent())
        assertNull(View(ContextWrapper(application)).lent())
        val activity = Robolectric.buildActivity(Activity::class.java).create().get()
        assertNull("an activity's own views are not lent one", View(activity).lent())
    }

    @Test
    fun lentAnActivityTheViewsContextIsThatActivitysAndThenTheApplicationsAgain() {
        val lendable = MutableContextWrapper(application)
        val view = View(SearchingContext(lendable) {})
        val activity = Robolectric.buildActivity(Activity::class.java).create().get()

        view.lent()?.baseContext = activity
        assertSame("what the view asks its context for now comes from the activity", activity, lendable.baseContext)
        assertSame(activity.windowManager, view.context.getSystemService(Context.WINDOW_SERVICE))

        view.lent()?.baseContext = application
        assertSame(application, lendable.baseContext)
    }
}
