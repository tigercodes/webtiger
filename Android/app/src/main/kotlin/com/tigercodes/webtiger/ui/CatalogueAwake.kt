// What is awake (056) in the design catalogue: the list under the switch it has in the window
// (057), with tabs in it, and with none. What a core would answer, with no core behind it.
package com.tigercodes.webtiger.ui

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AwakePage
import com.tigercodes.webtiger.core.v1.AwakeTab
import com.tigercodes.webtiger.core.v1.KeepAwake
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope

/** One state of the list of what is awake (056). */
internal enum class CatalogueAwake(@StringRes val title: Int) {
    LIST(R.string.catalogue_state_awake_list),
    EMPTY(R.string.catalogue_state_awake_empty),
    ;

    /**
     * What a core would answer for the state. With tabs: the one on screen, which is also on
     * its way; one behind it with nothing to keep it awake; one that plays a sound and was
     * typed into, whose name is longer than the list is wide; one in another Space that asks a
     * question; one with no name, downloading; and a Site in a Space with a long name, using
     * the camera. Two more are awake in private windows, and twelve are asleep. With none: no
     * tab is awake, and forty are asleep.
     */
    internal val page: AwakePage
        get() =
            if (this == LIST) {
                AwakePage(
                    tabs =
                        listOf(
                            tab(1, "Example Domain", "example.com", space = "", inSight = true, reasons = listOf(KeepAwake.KEEP_AWAKE_LOADING)),
                            tab(2, "A Story", "example.org", space = ""),
                            tab(
                                3,
                                "A page whose title is longer than the list is wide, and then goes on",
                                "music.example.com",
                                space = "",
                                reasons = listOf(KeepAwake.KEEP_AWAKE_SOUND, KeepAwake.KEEP_AWAKE_TYPED),
                            ),
                            tab(4, "Calendar", "calendar.example.com", space = "Work", reasons = listOf(KeepAwake.KEEP_AWAKE_QUESTION)),
                            tab(5, "", "files.example.net", space = "Work", reasons = listOf(KeepAwake.KEEP_AWAKE_DOWNLOAD)),
                            tab(
                                6,
                                "Meeting",
                                "meet.example.com",
                                space = "A Space with a long name of its own",
                                kept = true,
                                reasons = listOf(KeepAwake.KEEP_AWAKE_CAPTURE),
                            ),
                        ),
                    private_tabs = 2,
                    asleep = 12,
                )
            } else {
                AwakePage(asleep = 40)
            }

    private fun tab(
        id: Long,
        title: String,
        host: String,
        space: String,
        inSight: Boolean = false,
        kept: Boolean = false,
        reasons: List<KeepAwake> = emptyList(),
    ): AwakeTab =
        AwakeTab(
            tab_id = id,
            window_id = 1,
            space_id = if (space.isEmpty()) 1 else 2,
            space_name = space,
            title = title,
            host = host,
            in_sight = inSight,
            kept = kept,
            keeps_awake = reasons,
        )

    /** A model over a core that answers with this state, and nothing else. */
    fun model(context: Context, scope: CoroutineScope): AwakeModel {
        val answer = page
        val core = ScriptedCore { request -> if (request.query_awake != null) Response(awake_page = answer) else Response(ack = Ack()) }
        return AwakeModel(BrowserSession(core, SystemWebViewEngine(context), scope), windowId = 1)
    }
}

/** What is awake in one state: [AwakeList] over a scripted core that answers with it, under the switch it has in the window. */
@Composable
internal fun CatalogueAwakeList(state: CatalogueAwake, modifier: Modifier = Modifier) {
    val context = LocalContext.current.applicationContext
    val scope = rememberCoroutineScope()
    val model = remember(state) { state.model(context, scope) }
    Box(modifier = modifier) {
        AwakeList(model, onDone = {}, switcher = { LibrarySwitch(LibraryPart.AWAKE) {} })
    }
}
