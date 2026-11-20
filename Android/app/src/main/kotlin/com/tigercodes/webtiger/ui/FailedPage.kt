// A page that failed to load, and, where the core says the load may be gone on with, the warning
// before going on past a certificate that is not trusted (026 §6).
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/**
 * What a failed page offers when the core says its load may be gone on with (026): the host
 * whose certificate was not trusted, whether the warning is open, and the two presses there are.
 * Nothing goes on by one press: the first shows the warning, and only the button under the
 * warning goes on.
 */
internal class GoingOn(val host: String, val warned: Boolean, val onAdvanced: () -> Unit, val onGoOn: () -> Unit)

/** A page that failed to load. The core gave a code; the words are the shell's. */
@Composable
internal fun FailedPage(code: String, onRetry: () -> Unit, goingOn: GoingOn? = null) {
    val resources = LocalResources.current
    val packageName = LocalContext.current.packageName
    Column(
        modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(Tokens.Space.xl).testTag("failed-page"),
        verticalArrangement = Arrangement.spacedBy(Tokens.Space.m, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_warning),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(44.dp),
        )
        Text(
            text = stringResource(R.string.error_title),
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
        )
        Text(
            text = errorWords(resources, packageName, code),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.testTag("failed-words"),
        )
        Button(onClick = onRetry, modifier = Modifier.testTag("retry")) {
            Text(text = stringResource(R.string.error_retry))
        }
        when {
            goingOn == null -> Unit
            goingOn.warned -> CertificateWarning(goingOn)
            else ->
                TextButton(onClick = goingOn.onAdvanced, modifier = Modifier.testTag("certificate-advanced")) {
                    Text(text = stringResource(R.string.certificate_advanced))
                }
        }
    }
}

/**
 * The warning before going on (026 §6): what cannot be confirmed and what could come of it, when
 * going on is reasonable all the same, and under it the one button that goes on, which says that
 * it is not safe. It is said aloud when it appears.
 */
@Composable
private fun CertificateWarning(goingOn: GoingOn) {
    Surface(
        shape = RoundedCornerShape(Tokens.Radius.tile),
        color = MaterialTheme.colorScheme.surfaceVariant,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
        modifier = Modifier.widthIn(max = 520.dp).testTag("certificate-warning"),
    ) {
        Column(
            modifier = Modifier.padding(Tokens.Space.l).semantics { liveRegion = LiveRegionMode.Polite },
            verticalArrangement = Arrangement.spacedBy(Tokens.Space.s),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = stringResource(R.string.certificate_warning_title),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
                modifier = Modifier.semantics { heading() },
            )
            Text(
                text = stringResource(R.string.certificate_warning, goingOn.host),
                textAlign = TextAlign.Center,
                modifier = Modifier.testTag("certificate-warning-words"),
            )
            Text(text = stringResource(R.string.certificate_warning_only_if), textAlign = TextAlign.Center)
            // Not the page's main button, and not coloured as one: going on is allowed, and is
            // not what the page suggests.
            OutlinedButton(onClick = goingOn.onGoOn, modifier = Modifier.testTag("certificate-go-on")) {
                Text(text = stringResource(R.string.certificate_proceed, goingOn.host), textAlign = TextAlign.Center)
            }
        }
    }
}
