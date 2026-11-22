// The app's one activity: a window over the process's session. Almost nothing is in it.
package com.tigercodes.webtiger

import android.content.Intent
import android.content.pm.ApplicationInfo
import android.os.Bundle
import android.view.KeyEvent
import android.view.ViewGroup
import android.webkit.MimeTypeMap
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.ComposeView
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.BrowserSession
import com.tigercodes.webtiger.ui.BrowserWindow
import com.tigercodes.webtiger.ui.CatalogueScreen
import com.tigercodes.webtiger.ui.FillableWindow
import com.tigercodes.webtiger.ui.LaunchOptions
import com.tigercodes.webtiger.ui.LocalWindowKeys
import com.tigercodes.webtiger.ui.StartFailed
import com.tigercodes.webtiger.ui.WindowKeys
import com.tigercodes.webtiger.webview.AppPermissions
import com.tigercodes.webtiger.webview.pickerTypes

class MainActivity : ComponentActivity() {
    /** The app's own permissions, while this window is their way to ask the system (014). */
    private var permissions: AppPermissions? = null

    /** The window's answer to a keyboard's keys (036), asked before anything in the window has a key. */
    private val keys = WindowKeys()

    /** What is told when the system's dialog has been answered: one question is up at a time. */
    private var answered: (() -> Unit)? = null

    /** Asks the system for permissions by name, with the system's own dialog. */
    private val asksTheSystem =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
            // Whatever the person said: what the app now holds is what counts.
            answered.also { answered = null }?.invoke()
        }

    /** What is given what the system's file picker picked (019): one picker is open at a time. */
    private var picked: ((List<String>) -> Unit)? = null

    /** The system's picker of documents, for one file and for several. */
    private val picksOne =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { file ->
            picked.also { picked = null }?.invoke(listOfNotNull(file?.toString()))
        }
    private val picksSeveral =
        registerForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { files ->
            picked.also { picked = null }?.invoke(files.map { it.toString() })
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val debuggable = applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0
        val launch = launchOptions(intent, debuggable)
        if (launch.catalogue) {
            // The design catalogue needs no core: none is started.
            setContent { WebTigerTheme { CatalogueScreen() } }
            return
        }
        val session = (application as WebTigerApp).session(launch)
        // A page the core lets use the camera, the microphone or the location gets it when the
        // system lets the app: this window is how the system is asked. Not in fake mode, whose
        // session has nothing to ask with.
        permissions =
            session.getOrNull()?.systemPermissions?.also { own ->
                own.asker = { names, done ->
                    answered = done
                    asksTheSystem.launch(names)
                }
            }
        // A page's file input opens the system's picker of documents: this window is how it
        // is started. Not in fake mode, whose session has no chooser to start it with.
        session.getOrNull()?.files?.picker = { asked, done ->
            picked = done
            val types = pickerTypes(asked.kinds, MimeTypeMap.getSingleton()::getMimeTypeFromExtension).toTypedArray()
            if (asked.several) picksSeveral.launch(types) else picksOne.launch(types)
        }
        val window = ViewModelProvider(this)[WindowHolder::class.java]
        // Started with what another app handed over, words to search for (028) or an address
        // to open (031): the core is asked once the window is open. Not when the window is
        // being made again from what it was: it was asked then.
        if (savedInstanceState == null) session.getOrNull()?.let { running -> handedOver(intent, window.model(running)) }
        // The window is drawn by Compose, inside a view of the app's own that lets the system's
        // Autofill reach the page it shows (032, ui/FillableWindow.kt).
        val drawn =
            ComposeView(this).apply {
                setContent {
                    WebTigerTheme {
                        CompositionLocalProvider(LocalWindowKeys provides keys) {
                            session.fold(
                                onSuccess = { running -> BrowserWindow(running, window.model(running)) },
                                onFailure = { failure -> StartFailed(WebTigerApp.startCode(failure)) },
                            )
                        }
                    }
                }
            }
        setContentView(FillableWindow(this).apply { addView(drawn, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)) })
    }

    /**
     * What another app handed over while the window is there, words (028) or an address (031):
     * the window comes forward, and the core is asked.
     */
    /** A keyboard's keys (036): the window is asked first, and what it answers nothing else has. */
    override fun dispatchKeyEvent(event: KeyEvent): Boolean = keys.handle(event) || super.dispatchKeyEvent(event)

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        val launch = launchOptions(this.intent, applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0)
        if (launch.catalogue) return
        (application as WebTigerApp).session(launch).getOrNull()?.let { running ->
            handedOver(intent, ViewModelProvider(this)[WindowHolder::class.java].model(running))
        }
    }

    override fun onDestroy() {
        // The window goes, and the system's dialog with it: nobody can be asked from here, and
        // a question that was up is answered by what the app holds.
        permissions?.asker = null
        permissions = null
        answered.also { answered = null }?.invoke()
        super.onDestroy()
    }

    companion object {
        /** `--ez webtiger.fake true`: fake mode (003 §7). Debuggable builds only. */
        const val EXTRA_FAKE = "webtiger.fake"

        /** `--es webtiger.fake.start <address>`: what every new window opens in fake mode. */
        const val EXTRA_FAKE_START = "webtiger.fake.start"

        /** `--ez webtiger.fake.first_run true`: in fake mode, nobody has chosen where searches go yet (020). */
        const val EXTRA_FAKE_FIRST_RUN = "webtiger.fake.first_run"

        /** The words of [ACTION_SEARCH_WORDS]. */
        const val EXTRA_WORDS = "webtiger.search.words"

        /**
         * Search for [EXTRA_WORDS] in a new tab (028): what [SearchWordsActivity] starts the
         * window with. The words are handed to the core as words to search for, and are never
         * anything else.
         */
        const val ACTION_SEARCH_WORDS = "com.tigercodes.webtiger.action.SEARCH_WORDS"

        /** Hands the words an intent carries to a window's model. An intent that carries none does nothing. */
        fun searchWords(intent: Intent?, model: BrowserModel) {
            if (intent?.action != ACTION_SEARCH_WORDS) return
            intent.getStringExtra(EXTRA_WORDS)?.takeIf { it.isNotBlank() }?.let(model::searchFor)
        }

        /**
         * How long an address from another app may be to be handed on (031). One that is longer
         * is not handed on at all: a part of an address is another address.
         */
        const val LONGEST_ADDRESS = 8_000

        /**
         * The address the system asks the window to show (031): a link pressed in another app.
         * As it was given. Null for an intent that is not the system's "view", or that names no
         * address, or one too long. Nothing else of the intent is read: not what else the other
         * app put in it, and not who sent it.
         */
        fun address(from: Intent?): String? {
            if (from?.action != Intent.ACTION_VIEW) return null
            return from.dataString?.takeIf { it.isNotBlank() && it.length <= LONGEST_ADDRESS }
        }

        /**
         * Hands the address an intent asks to be shown to a window's model, which asks the core:
         * whether it is one to open is the core's to say, and nothing is made of it here.
         */
        fun openAddress(intent: Intent?, model: BrowserModel) {
            address(intent)?.let(model::openFromOutside)
        }

        /**
         * Hands a window's model what an intent carries from another app: words, or an address.
         * Not when the system is making the window again from the list of recent apps: the
         * intent it brings then is the one the window was first started with, and what that
         * carried was handed over the first time.
         */
        fun handedOver(intent: Intent?, model: BrowserModel) {
            if (intent != null && intent.flags and Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY != 0) return
            searchWords(intent, model)
            openAddress(intent, model)
        }

        /** `--ez webtiger.catalogue true`: the design catalogue instead of a browser (004). Debuggable builds only. */
        const val EXTRA_CATALOGUE = "webtiger.catalogue"

        /**
         * What a launch asked for. Fake mode and the catalogue are honoured in a debuggable build
         * only: an intent from another app must not be able to start a shipped WebTiger without
         * its data, or as something other than a browser.
         */
        fun launchOptions(intent: Intent?, debuggable: Boolean): LaunchOptions {
            if (!debuggable || intent == null) return LaunchOptions()
            val catalogue = intent.getBooleanExtra(EXTRA_CATALOGUE, false)
            if (!intent.getBooleanExtra(EXTRA_FAKE, false)) return LaunchOptions(catalogue = catalogue)
            return LaunchOptions(
                fake = true,
                fakeStart = intent.getStringExtra(EXTRA_FAKE_START).orEmpty(),
                catalogue = catalogue,
                firstRun = intent.getBooleanExtra(EXTRA_FAKE_FIRST_RUN, false),
            )
        }
    }
}

/**
 * Keeps the window's model while the activity is made again, and closes the window in the core
 * when the activity is gone for good.
 */
class WindowHolder : ViewModel() {
    private var model: BrowserModel? = null

    fun model(session: BrowserSession): BrowserModel = model ?: BrowserModel(session).also { model = it }

    override fun onCleared() {
        model?.close()
    }
}
