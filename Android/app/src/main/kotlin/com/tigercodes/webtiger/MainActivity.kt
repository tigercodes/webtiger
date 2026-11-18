// The app's one activity: a window over the process's session. Almost nothing is in it.
package com.tigercodes.webtiger

import android.content.Intent
import android.content.pm.ApplicationInfo
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.BrowserSession
import com.tigercodes.webtiger.ui.BrowserWindow
import com.tigercodes.webtiger.ui.CatalogueScreen
import com.tigercodes.webtiger.ui.LaunchOptions
import com.tigercodes.webtiger.ui.StartFailed

class MainActivity : ComponentActivity() {
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
