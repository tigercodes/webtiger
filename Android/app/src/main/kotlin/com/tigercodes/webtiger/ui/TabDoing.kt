// Marks for what a tab is doing (058): of the core's reasons a tab is kept awake (052), the two
// that have a mark in the row. The others are said in words in the list of what is awake (056).
package com.tigercodes.webtiger.ui

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.KeepAwake
import com.tigercodes.webtiger.core.v1.TabSummary

/**
 * What a tab is doing that a person must be able to find at a glance: its page holds the camera
 * or the microphone, or plays a sound. One mark to a tab: the camera or the microphone before
 * sound. The core says the reasons, and which of them hold; the shell chooses a picture.
 */
internal enum class TabDoing(@DrawableRes val picture: Int, @StringRes val words: Int, val tag: String) {
    CAPTURE(R.drawable.ic_capture, R.string.awake_why_capture, "doing-capture"),
    SOUND(R.drawable.ic_sound, R.string.awake_why_sound, "doing-sound"),
    ;

    companion object {
        /** The mark a tab has; none for a tab that does neither. */
        fun of(tab: TabSummary): TabDoing? =
            when {
                KeepAwake.KEEP_AWAKE_CAPTURE in tab.keeps_awake -> CAPTURE
                KeepAwake.KEEP_AWAKE_SOUND in tab.keeps_awake -> SOUND
                else -> null
            }
    }
}
