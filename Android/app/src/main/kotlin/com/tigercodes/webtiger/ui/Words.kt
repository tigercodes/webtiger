// The core speaks codes; the shell speaks words (Localization/strings.json).
package com.tigercodes.webtiger.ui

import android.annotation.SuppressLint
import android.content.res.Resources
import com.tigercodes.webtiger.R

private val code = Regex("^[a-z0-9_]+(\\.[a-z0-9_]+)+$")

/**
 * The words for a core error code: the string keyed `error.<code>` (the resource
 * `error_net_dns` for "net.dns"), or the general ones when this build has none for it. A
 * code's own text is never shown.
 */
@SuppressLint("DiscouragedApi") // The codes are the core's, so the name is known only when one arrives; res/raw/keep.xml keeps the strings.
internal fun errorWords(resources: Resources, packageName: String, errorCode: String): String {
    val id = if (code.matches(errorCode)) resources.getIdentifier("error_" + errorCode.replace('.', '_'), "string", packageName) else 0
    return resources.getString(if (id != 0) id else R.string.error_net_failed)
}
