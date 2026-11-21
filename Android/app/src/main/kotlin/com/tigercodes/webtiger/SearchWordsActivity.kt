// "Search with WebTiger" in other apps' menu of selected text (028).
//
// The system offers every app's "process this text" in the bar that selecting text gives. This
// is WebTiger's: an activity other apps can start, which shows nothing. It reads the words,
// brings the app's window forward with them (or starts it), and is gone. What the words are,
// and where the search goes, is the core's to say: here they are only carried.
//
// Why not the window itself: the menu starts its item inside the other app's task, and the
// window started there would be a second WebTiger window, living in another app.
package com.tigercodes.webtiger

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle

class SearchWordsActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        words(intent)?.let { startActivity(windowIntent(this, it)) }
        // Nothing is handed back: the selected words stay as they are.
        finish()
    }

    companion object {
        /** How many characters are carried at the most. The core cuts what is searched for much shorter (023): this only keeps what is handed on small. */
        const val MOST_CARRIED = 4_000

        /**
         * The words an intent carries to be searched for. Null for an intent that is not the
         * system's "process this text", or that carries no words.
         */
        fun words(from: Intent?): String? {
            if (from?.action != Intent.ACTION_PROCESS_TEXT) return null
            return from.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)?.toString()?.take(MOST_CARRIED)?.takeIf { it.isNotBlank() }
        }

        /**
         * What starts the app's window with words to search for, in the window's own task and
         * not in the task of the app the words came from. The window is one (the manifest's
         * `singleTask`): the one there is comes forward and is told of them.
         */
        fun windowIntent(context: Context, words: String): Intent =
            Intent(context, MainActivity::class.java)
                .setAction(MainActivity.ACTION_SEARCH_WORDS)
                .putExtra(MainActivity.EXTRA_WORDS, words)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }
}
