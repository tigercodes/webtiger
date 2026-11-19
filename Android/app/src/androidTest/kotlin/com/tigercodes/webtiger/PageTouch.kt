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
     * Where an element is in a WebView: its middle in the view's own pixels, with the view's
     * size, read together. Null when the page has no such element or the view has no size.
     */
    private fun place(view: WebView, element: String): List<Float>? {
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
        val page = evaluate(view, script).orEmpty().split(' ').mapNotNull { it.toFloatOrNull() }
        if (page.size != 4 || page[2] <= 0 || page[3] <= 0) return null
        var size = listOf(0f, 0f)
        InstrumentationRegistry.getInstrumentation().runOnMainSync { size = listOf(view.width.toFloat(), view.height.toFloat()) }
        if (size[0] <= 0 || size[1] <= 0) return null
        return listOf(page[0] / page[2] * size[0], page[1] / page[3] * size[1], size[0], size[1])
    }

    /**
     * Taps the element with the id `element` in the page a WebView shows: a touch down and up
     * at its middle, given to the WebView as the system gives a finger's. The WebView must be
     * in a window on the screen: a page that is not shown is sent no touches.
     *
     * The touch waits until the element is where it was a moment before. A window that has
     * just opened may still be turning with the device, or laying its page out, and a touch
     * aimed at where the element was lands on nothing: one run in several did, straight after
     * the test that turns the device.
     */
    fun tap(view: WebView, element: String) {
        awaitShown(view)
        var before: List<Float>? = null
        repeat(100) {
            val now = place(view, element)
            if (now != null && now == before) {
                touch(view, now[0], now[1])
                return
            }
            before = now
            Thread.sleep(100)
        }
        throw AssertionError(if (before == null) "no element #$element in the page" else "the page never came to rest")
    }

    private fun touch(view: WebView, x: Float, y: Float) {
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
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
