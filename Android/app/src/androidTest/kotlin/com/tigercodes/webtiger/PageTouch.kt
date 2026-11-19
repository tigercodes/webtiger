// A finger on a page, for the tests on a device. A page is given a new window only when somebody
// touched it (006), and a WebView takes nothing a test's script does for a touch: so the tests
// touch.
package com.tigercodes.webtiger

import android.os.SystemClock
import android.view.InputDevice
import android.view.MotionEvent
import android.webkit.WebView
import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit

internal object PageTouch {
    /** Evaluates an expression that gives a string in the page a WebView shows. Null when it gave none. */
    fun evaluate(view: WebView, script: String): String? {
        val answer = CompletableFuture<String?>()
        InstrumentationRegistry.getInstrumentation().runOnMainSync { view.evaluateJavascript(script) { json -> answer.complete(json) } }
        val json = answer.get(10, TimeUnit.SECONDS) ?: return null
        // A string comes back as JSON: quoted, with its quotes and backslashes escaped.
        if (json.length < 2 || !json.startsWith("\"")) return null
        return json.substring(1, json.length - 1).replace("\\\"", "\"").replace("\\\\", "\\")
    }

    /** Waits until a WebView is in a window, laid out, and its page says it can be seen. */
    fun awaitShown(view: WebView) {
        repeat(500) {
            var laidOut = false
            InstrumentationRegistry.getInstrumentation().runOnMainSync { laidOut = view.isAttachedToWindow && view.width > 0 && view.height > 0 }
            if (laidOut && evaluate(view, "document.visibilityState") == "visible") return
            Thread.sleep(20)
        }
        throw AssertionError("the web view was never shown")
    }

    /**
     * Taps the element with the id `element` in the page a WebView shows: a touch down and up
     * at its middle, given to the WebView as the system gives a finger's. The WebView must be
     * in a window on the screen: a page that is not shown is sent no touches.
     */
    fun tap(view: WebView, element: String) {
        awaitShown(view)
        // Where the element is, in the page's own measure, and how much of the page the view shows.
        val script =
            """
            (() => {
                const element = document.getElementById('$element');
                if (!element) return '';
                const box = element.getBoundingClientRect();
                return [box.left + box.width / 2, box.top + box.height / 2, window.innerWidth, window.innerHeight].join(' ');
            })()
            """.trimIndent()
        val place = evaluate(view, script).orEmpty().split(' ').mapNotNull { it.toFloatOrNull() }
        check(place.size == 4 && place[2] > 0 && place[3] > 0) { "no element #$element in the page" }
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            val x = place[0] / place[2] * view.width
            val y = place[1] / place[3] * view.height
            val down = SystemClock.uptimeMillis()
            for ((action, time) in listOf(MotionEvent.ACTION_DOWN to down, MotionEvent.ACTION_UP to down + 40)) {
                val event = MotionEvent.obtain(down, time, action, x, y, 0)
                event.source = InputDevice.SOURCE_TOUCHSCREEN
                view.dispatchTouchEvent(event)
                event.recycle()
            }
        }
    }
}
