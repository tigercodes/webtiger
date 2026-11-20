// The Permissions list (014 §7): the sites that were allowed to use the camera, the microphone
// or the location, each with what it may use, and a button to take that back. Taking it back
// stops nothing that is running: the site asks again the next time.
package com.tigercodes.webtiger.ui

import android.icu.text.ListFormatter
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.SitePermissions
import com.tigercodes.webtiger.design.Tokens

/**
 * The list over `model`. `onDone` is called when a person is finished. With a `switcher`, it
 * stands where the title would, on a row of its own: the list is then one of several a person
 * can choose between.
 */
@Composable
fun PermissionsList(
    model: PermissionsModel,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    switcher: (@Composable () -> Unit)? = null,
) {
    LaunchedEffect(model) { model.reload() }
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = Modifier.fillMaxSize()) {
            ListHeader(switcher = switcher, title = R.string.permissions_title) {
                TextButton(onClick = model::forgetEverything, enabled = model.sites.isNotEmpty(), modifier = Modifier.testTag("permissions-clear")) {
                    Text(stringResource(R.string.permissions_clear))
                }
                TextButton(onClick = onDone, modifier = Modifier.testTag("permissions-done")) { Text(stringResource(R.string.permissions_done)) }
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            when {
                // Nothing is said until the core has answered: not that no site was allowed anything.
                !model.loaded -> Unit
                model.sites.isEmpty() ->
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(
                            text = stringResource(R.string.permissions_empty),
                            textAlign = TextAlign.Center,
                            modifier = Modifier.padding(Tokens.Space.l).testTag("permissions-nothing"),
                        )
                    }
                else ->
                    LazyColumn(modifier = Modifier.fillMaxSize()) {
                        items(model.sites, key = { it.origin }) { site ->
                            SiteRow(site = site, onForget = { model.forget(site) })
                            HorizontalDivider(color = Tokens.Palette.hairline.current, modifier = Modifier.padding(start = Tokens.Space.m))
                        }
                    }
            }
        }
    }
}

/** One site: its name, as the core names it, and what it may use; a button takes that back. */
@Composable
private fun SiteRow(site: SitePermissions, onForget: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m), verticalAlignment = Alignment.CenterVertically) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .padding(vertical = Tokens.Space.xs)
                    // One thing to a screen reader: the site and what it may use.
                    .semantics(mergeDescendants = true) {}
                    .testTag("permission-site"),
            verticalArrangement = Arrangement.Center,
        ) {
            // A name may be any length: it is cut in its middle, where its start and its ending both stay.
            Text(text = site.site, maxLines = 2, overflow = TextOverflow.MiddleEllipsis)
            Text(text = allowedWords(site), style = MaterialTheme.typography.bodySmall)
        }
        IconButton(onClick = onForget, modifier = Modifier.testTag("permission-forget")) {
            Icon(painter = painterResource(R.drawable.ic_delete), contentDescription = stringResource(R.string.permissions_forget))
        }
    }
}

/**
 * What a site may use, in words, joined as the person's language joins a list. A kind this
 * build has no words for is left out.
 */
@Composable
internal fun allowedWords(site: SitePermissions): String {
    val names =
        site.kinds.mapNotNull { kind ->
            when (kind) {
                PermissionKind.PERMISSION_KIND_CAMERA -> stringResource(R.string.permissions_kind_camera)
                PermissionKind.PERMISSION_KIND_MICROPHONE -> stringResource(R.string.permissions_kind_microphone)
                PermissionKind.PERMISSION_KIND_LOCATION -> stringResource(R.string.permissions_kind_location)
                else -> null
            }
        }
    return ListFormatter.getInstance(LocalConfiguration.current.locales[0]).format(names)
}
