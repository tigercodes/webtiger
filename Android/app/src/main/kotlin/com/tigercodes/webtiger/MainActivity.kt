// The app's one activity: a window over the process's session. Almost nothing is in it.
package com.tigercodes.webtiger

import android.content.Intent
import android.content.pm.ApplicationInfo
import android.os.Bundle
import android.webkit.MimeTypeMap
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.BrowserSession
import com.tigercodes.webtiger.ui.BrowserWindow
import com.tigercodes.webtiger.ui.CatalogueScreen
import com.tigercodes.webtiger.ui.LaunchOptions
import com.tigercodes.webtiger.ui.StartFailed
import com.tigercodes.webtiger.webview.AppPermissions
import com.tigercodes.webtiger.webview.pickerTypes

class MainActivity : ComponentActivity() {
    /** The app's own permissions, while this window is their way to ask the system (014). */
    private var permissions: AppPermissions? = null

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
        setContent {
            WebTigerTheme {
                session.fold(
                    onSuccess = { running -> BrowserWindow(running, window.model(running)) },
                    onFailure = { failure -> StartFailed(WebTigerApp.startCode(failure)) },
                )
            }
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
            return LaunchOptions(fake = true, fakeStart = intent.getStringExtra(EXTRA_FAKE_START).orEmpty(), catalogue = catalogue)
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
