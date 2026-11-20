// A page asks for a file (019): `<input type="file">`, on the System WebView. The WebView picks
// nothing by itself: left alone, the button does nothing. The adapter asks the engine's file
// chooser and gives the WebView what was picked.
//
// The chooser is a seam (019 §3): an engine picks nothing, and shows nothing of the system's,
// unless the app itself gave it the way to the system's picker.
package com.tigercodes.webtiger.webview

/** What a page asks of a file chooser: whether several files may be chosen, and the kinds it says it accepts. */
class FilesAsked(val several: Boolean, val kinds: List<String> = emptyList())

/** How an engine picks files for a page. `done` is told the files' addresses, or none when nothing is chosen. */
fun interface FileChooser {
    fun choose(asked: FilesAsked, done: (List<String>) -> Unit)
}

/**
 * Picks nothing, and shows nothing. What every engine has unless it is given another: so every
 * engine a test makes, and fake mode's. The system's picker is another app's screen, and
 * nothing but the app itself, started for real, may start it.
 */
object NoFiles : FileChooser {
    override fun choose(asked: FilesAsked, done: (List<String>) -> Unit) = done(emptyList())
}

/**
 * The system's document picker, while a window gives the way to start it. With no window, nothing
 * is picked.
 */
class AppFiles : FileChooser {
    /** How the system's picker is started; it calls back with what was picked. Null while there is no window to start it from. */
    var picker: ((FilesAsked, (List<String>) -> Unit) -> Unit)? = null

    override fun choose(asked: FilesAsked, done: (List<String>) -> Unit) {
        val start = picker
        if (start == null) done(emptyList()) else start(asked, done)
    }
}

/**
 * The kinds of file a page says it accepts, as the system's picker takes them: media types,
 * one kind of picture or every kind of picture. A page may name a kind by an ending instead
 * (`.txt`): `typeOf` says the type the system knows that ending by, or null. With nothing the
 * picker can go by, every file is offered: what a page accepts narrows what is shown, and
 * promises nothing.
 */
fun pickerTypes(kinds: List<String>, typeOf: (String) -> String?): List<String> =
    kinds
        .mapNotNull { kind ->
            val named = kind.trim().lowercase()
            when {
                named.startsWith(".") -> typeOf(named.drop(1))
                named.contains("/") -> named
                else -> null
            }
        }.distinct()
        .ifEmpty { listOf("*/*") }

/** The web views of one engine whose pages have a chooser open. Used on the main thread, as the engine is. */
internal class Files(private val chooser: FileChooser) {
    private class Open(val number: Long, val give: (List<String>?) -> Unit)

    private val open = HashMap<Long, Open>()
    private var opened = 0L

    /**
     * Picks files for a page of `webViewId`, and gives them, once: the files' addresses, or
     * null when nothing is to be given. Nothing is: when nothing was picked; and at once, when
     * a chooser is open for that web view already. A page that may have one file is given no
     * more than the first.
     */
    fun choose(webViewId: Long, asked: FilesAsked, give: (List<String>?) -> Unit) {
        if (webViewId in open) {
            give(null)
            return
        }
        opened += 1
        val mine = opened
        open[webViewId] = Open(mine, give)
        chooser.choose(asked) { picked ->
            // The web view went while the chooser was open: it was told nothing then, and
            // what was picked goes nowhere.
            if (open[webViewId]?.number == mine) {
                open.remove(webViewId)
                val given = if (asked.several) picked else picked.take(1)
                give(given.ifEmpty { null })
            }
        }
    }

    /** A web view is going: its page is told that nothing was chosen. */
    fun dropped(webViewId: Long) {
        open.remove(webViewId)?.give?.invoke(null)
    }
}
