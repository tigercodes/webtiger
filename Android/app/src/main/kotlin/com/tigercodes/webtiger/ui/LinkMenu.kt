// A link's menu (018) and a picture's (022): what the window shows when the person presses and
// holds a link or a picture of the page. Where the link leads, or the picture's address, and
// what can be done with it: opened in a new tab or saved, which are the core's to do, or
// copied, which is the shell's. A picture that is a link has both. There is no preview: a
// preview loads a page nobody chose to load.
package com.tigercodes.webtiger.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/**
 * What the person pressed and held: the web view whose page has it, where the link leads, and
 * the picture's address. `url` is empty for a picture that is no link, `picture` for a link in
 * words.
 */
internal data class PressedLink(val webViewId: Long, val url: String, val picture: String = "")

/** What a choice in the menu does. */
internal class LinkChoices(
    val open: () -> Unit,
    val download: () -> Unit,
    val copy: () -> Unit,
    val openPicture: () -> Unit,
    val downloadPicture: () -> Unit,
    val dismiss: () -> Unit,
)

/**
 * Where "Copy Link" puts an address. The system's clipboard is the person's own: in fake mode,
 * and in every session a test makes, a link is copied to a clipboard of the session's instead,
 * so that no test changes what somebody had copied.
 */
fun interface LinkClipboard {
    fun copy(text: String)
}

/** The system's clipboard. */
class SystemClipboard(private val context: Context) : LinkClipboard {
    override fun copy(text: String) {
        context.getSystemService(ClipboardManager::class.java)?.setPrimaryClip(ClipData.newPlainText("", text))
    }
}

/** A clipboard of the session's own, which keeps what was last copied and shows it to nobody. */
class MemoryClipboard : LinkClipboard {
    var copied: String? = null
        private set

    override fun copy(text: String) {
        copied = text
    }
}

/**
 * The menu, over the page and at the foot of it. A touch beside it puts it away and is not
 * the page's; so does the system's Back.
 */
@Composable
internal fun LinkMenu(link: PressedLink, choices: LinkChoices) {
    val onDismiss = choices.dismiss
    BackHandler(onBack = onDismiss)
    Box(modifier = Modifier.fillMaxSize()) {
        // The page under it is not pressed meanwhile. This is not offered to assistive
        // services as something to press: "Cancel" is.
        Box(modifier = Modifier.fillMaxSize().pointerInput(Unit) { detectTapGestures { onDismiss() } }.testTag("link-menu-beside"))
        val title = stringResource(if (link.url.isEmpty()) R.string.link_image_menu else R.string.link_menu)
        Column(
            modifier =
                Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .background(Tokens.Palette.fieldFill.current)
                    // A touch on the menu itself, between its items, is not one beside it.
                    .pointerInput(Unit) { detectTapGestures { } }
                    .semantics { paneTitle = title }
                    .testTag("link-menu")
                    // A picture that is a link has more choices than a short window has room
                    // for (a phone held sideways): the menu scrolls, so that the last of them,
                    // and "Cancel", can always be reached.
                    .verticalScroll(rememberScrollState()),
        ) {
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            // Where the link leads, as the engine says it: its beginning, which names the site.
            // For a picture that is no link, the picture's own address.
            Text(
                text = link.url.ifEmpty { link.picture },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m, end = Tokens.Space.m, top = Tokens.Space.s).testTag("link-address"),
            )
            if (link.url.isNotEmpty()) {
                Item(R.string.link_open_new_tab, "link-open", choices.open)
                Item(R.string.link_download, "link-download", choices.download)
                Item(R.string.link_copy, "link-copy", choices.copy)
            }
            if (link.picture.isNotEmpty()) {
                Item(R.string.link_open_image_new_tab, "picture-open", choices.openPicture)
                Item(R.string.link_download_image, "picture-download", choices.downloadPicture)
            }
            Item(R.string.link_cancel, "link-cancel", onDismiss)
            Spacer(modifier = Modifier.height(Tokens.Space.xs))
        }
    }
}

@Composable
private fun Item(words: Int, tag: String, onChosen: () -> Unit) {
    TextButton(onClick = onChosen, modifier = Modifier.fillMaxWidth().padding(horizontal = Tokens.Space.xs).testTag(tag)) {
        Text(text = stringResource(words), modifier = Modifier.fillMaxWidth())
    }
}
