// The core speaks codes; the shell speaks words (Localization/strings.json).
package com.tigercodes.webtiger.ui

import android.annotation.SuppressLint
import android.content.res.Resources
import com.tigercodes.webtiger.R

private val code = Regex("^[a-z0-9_]+(\\.[a-z0-9_]+)+$")

/**
 * The words for a core error code: the string keyed `error.<code>` (the resource
 * `error_net_dns` for "net.dns"). A code with none of its own has those of the code above it
 * (`net.tls` for `net.tls.date`, 021 §3), so that a newer core's finer code is never worse
 * than the one it refines; and only then the general ones. A code's own text is never shown.
 */
@SuppressLint("DiscouragedApi") // The codes are the core's, so the name is known only when one arrives; res/raw/keep.xml keeps the strings.
internal fun errorWords(resources: Resources, packageName: String, errorCode: String): String {
    if (code.matches(errorCode)) {
        val parts = errorCode.split('.')
        for (length in parts.size downTo 2) {
            val id = resources.getIdentifier("error_" + parts.take(length).joinToString("_"), "string", packageName)
            if (id != 0) return resources.getString(id)
        }
    }
    return resources.getString(R.string.error_net_failed)
}

/**
 * The words for why a Space was not made, or not named (024): the code's own
 * (`error.space.too_many`), or `unknown`, what is said when no more is known. Never the code's
 * own text, and never what is said of a page that did not load.
 */
@SuppressLint("DiscouragedApi") // As above: the codes are the core's.
internal fun spaceRefusalWords(resources: Resources, packageName: String, errorCode: String, unknown: Int = R.string.error_space_failed): String {
    if (code.matches(errorCode)) {
        val id = resources.getIdentifier("error_" + errorCode.replace('.', '_'), "string", packageName)
        if (id != 0) return resources.getString(id)
    }
    return resources.getString(unknown)
}
