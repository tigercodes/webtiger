// A finger on a page, for the tests on a device. A page is given a new window only when somebody
// touched it (006), and a WebView takes nothing a test's script does for a touch: so the tests
// touch.
package com.tigercodes.webtiger

import android.os.SystemClock
import android.view.InputDevice
import android.view.MotionEvent
import android.view.ViewConfiguration
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
     *
     * By the part of the page that is on the screen (its visual viewport), not by the width
     * the page was laid out at. A page with no viewport of its own is laid out 980 wide; the
     * WebView usually shows all of that, shrunk to fit, and now and then shows it at its own
     * size, 412 of the 980 across. Measured against the laid-out width, a touch then landed
     * up and to the left of what it was aimed at: on the page's heading, not its link (seen:
     * about one run in four of a test that loads such a page twice).
     */
    private fun place(view: WebView, element: String): List<Float>? {
        // Where the element is in what the view shows of the page, and how much that is.
        val script =
            """
            (() => {
                const element = document.getElementById('$element');
                if (!element) return '';
                const box = element.getBoundingClientRect();
                const shown = window.visualViewport;
                return [box.left + box.width / 2 - shown.offsetLeft, box.top + box.height / 2 - shown.offsetTop, shown.width, shown.height].join(' ');
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
        val (x, y) = atRest(view, element)
        touch(view, x, y)
    }

    /**
     * Presses and holds the element with the id `element`: a finger put down at its middle,
     * left there for as long as the system takes a press to be a long one and a little more,
     * and lifted. The wait is the test's own thread's: the WebView's must be free to notice.
     */
    fun longPress(view: WebView, element: String) {
        val (x, y) = atRest(view, element)
        val down = SystemClock.uptimeMillis()
        InstrumentationRegistry.getInstrumentation().runOnMainSync { send(view, down, down, MotionEvent.ACTION_DOWN, x, y) }
        Thread.sleep(ViewConfiguration.getLongPressTimeout() + 400L)
        InstrumentationRegistry.getInstrumentation().runOnMainSync { send(view, down, SystemClock.uptimeMillis(), MotionEvent.ACTION_UP, x, y) }
    }

    /** The middle of an element, once it is where it was a moment before. */
    private fun atRest(view: WebView, element: String): Pair<Float, Float> {
        awaitShown(view)
        var before: List<Float>? = null
        repeat(100) {
            val now = place(view, element)
            if (now != null && now == before) return now[0] to now[1]
            before = now
            Thread.sleep(100)
        }
        throw AssertionError(if (before == null) "no element #$element in the page" else "the page never came to rest")
    }

    private fun touch(view: WebView, x: Float, y: Float) {
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            val down = SystemClock.uptimeMillis()
            send(view, down, down, MotionEvent.ACTION_DOWN, x, y)
            send(view, down, down + 40, MotionEvent.ACTION_UP, x, y)
        }
    }

    /** One event of a finger's, given to the WebView as the system gives it. On the main thread. */
    private fun send(view: WebView, down: Long, time: Long, action: Int, x: Float, y: Float) {
        val event = MotionEvent.obtain(down, time, action, x, y, 0)
        event.source = InputDevice.SOURCE_TOUCHSCREEN
        view.dispatchTouchEvent(event)
        event.recycle()
    }
}
