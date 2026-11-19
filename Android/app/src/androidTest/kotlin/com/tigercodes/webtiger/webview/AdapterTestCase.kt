// What the adapter suites share on a device: a loopback site, a scripted core, the engine under
// test over the device's real WebView, and helpers that speak the WebViews host service as the
// core does. The Apple suite's AdapterTestCase.swift, for Android.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.view.ViewGroup
import android.webkit.WebView
import androidx.activity.ComponentActivity
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import com.tigercodes.webtiger.PageTouch
import com.tigercodes.webtiger.TestSite
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.LoginSet
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.NewWindowRequested
import com.tigercodes.webtiger.core.v1.WebViewCommand
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewDestroy
import com.tigercodes.webtiger.core.v1.WebViewLoad
import java.util.UUID
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.junit.After
import org.junit.Before

internal abstract class AdapterTestCase {
    protected val site = TestSite()
    protected val core = ScriptedCore()
    protected lateinit var engine: SystemWebViewEngine
    private val created = ArrayList<Long>()

    /** A window on the screen for the web views a test touches, opened when one first is. */
    private var stage: ActivityScenario<ComponentActivity>? = null

    @Before
    fun attach() {
        onMain {
            engine = SystemWebViewEngine(ApplicationProvider.getApplicationContext<Context>())
            engine.attach(core)
        }
    }

    @After
    fun release() {
        for (id in created.toList()) destroy(id)
        stage?.close()
        // The login sets' profiles stay behind: the WebView deletes none that was used since the
        // app started (SystemWebViewIsolationTest). They go with the app's data, which the
        // runner removes when it uninstalls the app after the run.
        site.close()
    }

    // Helpers

    /** Runs `block` on the main thread, where a WebView must be used, and gives back its result. */
    protected fun <T> onMain(block: () -> T): T {
        var result: Result<T>? = null
        InstrumentationRegistry.getInstrumentation().runOnMainSync { result = runCatching(block) }
        return checkNotNull(result).getOrThrow()
    }

    /** Asks the engine as the core does: on the main thread, one request after the other. */
    protected fun answer(request: HostRequest): HostResponse = runBlocking { withContext(Dispatchers.Main) { engine.answer(request) } }

    protected fun newLoginSet(): String = UUID.randomUUID().toString()

    protected fun create(
        id: Long,
        loginSet: String = newLoginSet(),
        ephemeral: Boolean = false,
        kind: EngineKind = EngineKind.ENGINE_KIND_CHROMIUM,
    ): HostResponse {
        val answer = answer(HostRequest(web_view_create = WebViewCreate(web_view_id = id, engine = kind, login_set = LoginSet(id = loginSet, ephemeral = ephemeral))))
        if (answer.ack != null) created.add(id)
        return answer
    }

    protected fun load(id: Long, url: String): HostResponse = answer(HostRequest(web_view_load = WebViewLoad(web_view_id = id, url = url)))

    protected fun command(id: Long, command: NavigationCommand): HostResponse =
        answer(HostRequest(web_view_command = WebViewCommand(web_view_id = id, command = command)))

    protected fun destroy(id: Long): HostResponse {
        created.remove(id)
        return answer(HostRequest(web_view_destroy = WebViewDestroy(web_view_id = id)))
    }

    /** The real WebView behind a web view the engine holds. */
    protected fun webView(id: Long): WebView? = onMain { engine.webView(id) as? WebView }

    /** Has a web view the engine made unasked destroyed with the others when the test ends. */
    protected fun track(id: Long) {
        created.add(id)
    }

    /**
     * Puts a web view in a window on the screen, as the tab in front is. A page that is not
     * shown is sent no touches, and a page nobody touched is given no window.
     */
    protected fun show(id: Long) {
        val scenario = stage ?: ActivityScenario.launch(ComponentActivity::class.java).also { stage = it }
        scenario.onActivity { activity ->
            val view = checkNotNull(engine.webView(id)) { "no web view $id" }
            if (view.parent !== activity.findViewById<ViewGroup>(android.R.id.content)) {
                (view.parent as? ViewGroup)?.removeView(view)
                activity.setContentView(view, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
            }
        }
        PageTouch.awaitShown(checkNotNull(webView(id)))
    }

    /** Taps an element of a web view's page with a finger, having put the web view on the screen. */
    protected fun tap(id: Long, element: String) {
        show(id)
        PageTouch.tap(checkNotNull(webView(id)), element)
    }

    /** The windows a web view's page has asked for, in order. */
    protected fun windowsAsked(by: Long): List<NewWindowRequested> = reports(by).mapNotNull { it.new_window_requested }

    /**
     * Waits for a web view's page to have asked for `count` windows, and has the last one
     * destroyed when the test ends. The last one asked for.
     */
    protected fun awaitWindow(askedBy: Long, count: Int = 1): NewWindowRequested {
        await(askedBy, "a window asked for") { windowsAsked(askedBy).size >= count }
        return windowsAsked(askedBy).last().also { track(it.new_web_view_id) }
    }

    /** What the adapter has reported about a web view, in order. */
    protected fun reports(id: Long): List<EngineEvent> = core.reports.filter { it.web_view_id == id }

    /** Waits until the reports about a web view satisfy `done`, for up to fifteen seconds. */
    protected fun await(id: Long, what: String, done: (List<EngineEvent>) -> Boolean) {
        repeat(1500) {
            if (done(reports(id))) return
            Thread.sleep(10)
        }
        throw AssertionError("timed out waiting for $what; reported: ${names(reports(id))}")
    }

    protected fun finished(reports: List<EngineEvent>, times: Int = 1): Boolean = reports.count { it.navigation_finished != null } >= times

    protected fun failed(reports: List<EngineEvent>): Boolean = reports.any { it.navigation_failed != null }

    protected fun lastTitle(id: Long): String = reports(id).lastOrNull { it.title_changed != null }?.title_changed?.title.orEmpty()

    protected fun hasTitle(title: String): (List<EngineEvent>) -> Boolean = { reports -> reports.any { it.title_changed?.title == title } }

    /** The addresses reported as the page's, by a commit or by the page changing its own. */
    protected fun addresses(id: Long): List<String> = reports(id).mapNotNull { it.navigation_committed?.url ?: it.address_changed?.url }

    /** The reports' kinds, without progress, which arrives at the engine's own pace. */
    protected fun names(reports: List<EngineEvent>): List<String> =
        reports.mapNotNull { report ->
            val failed = report.navigation_failed
            when {
                report.navigation_started != null -> "started"
                report.navigation_committed != null -> "committed"
                report.navigation_finished != null -> "finished"
                failed != null -> "failed(${failed.code})"
                report.address_changed != null -> "address"
                report.title_changed != null -> "title"
                report.history_changed != null -> "history"
                report.renderer_gone != null -> "gone"
                report.new_window_requested != null -> "window"
                report.close_requested != null -> "close"
                report.outside_link_requested != null -> "outside"
                else -> null
            }
        }

    /**
     * Runs the body of an async function in a web view's page and gives back what it returns,
     * as JSON. Null when the script threw or never answered.
     */
    protected fun run(id: Long, body: String): String? {
        val view = webView(id) ?: return null
        val script =
            """
            (() => {
                window.__webtiger = undefined;
                (async () => { $body })().then(
                    value => { window.__webtiger = JSON.stringify({ value: value === undefined ? null : value }); },
                    error => { window.__webtiger = JSON.stringify({ error: String(error) }); });
            })();
            """.trimIndent()
        evaluate(view, script)
        repeat(500) {
            val answer = evaluate(view, "window.__webtiger === undefined ? '' : window.__webtiger")
            if (!answer.isNullOrEmpty()) {
                return Regex("^\\{\"value\":(.*)\\}$").find(answer)?.groupValues?.get(1)
            }
            Thread.sleep(20)
        }
        return null
    }

    /** Evaluates an expression in a page; what it gives is a string, or null. */
    private fun evaluate(view: WebView, script: String): String? {
        val answer = CompletableFuture<String?>()
        onMain { view.evaluateJavascript(script) { json -> answer.complete(json) } }
        val json = answer.get(10, TimeUnit.SECONDS) ?: return null
        // A string comes back as JSON: quoted, with its quotes and backslashes escaped.
        if (json.length < 2 || !json.startsWith("\"")) return null
        return json.substring(1, json.length - 1).replace("\\\"", "\"").replace("\\\\", "\\")
    }
}
