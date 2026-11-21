// What the core asks of the shell (000 §5.5), sorted by who answers: the System service is the
// shell's own, and everything else is a web engine's.
package com.tigercodes.webtiger.ui

import android.content.ActivityNotFoundException
import android.content.ContentResolver
import android.content.ContentUris
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.MediaStore
import com.tigercodes.webtiger.core.HostServices
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse

/**
 * Opens an address with whatever the system has for it: the mail app for a letter, the dialler
 * for a number. True when the system took it; false when nothing on the device opens it.
 *
 * The shell opens only what the core asks it to (`OpenOutside`): which addresses may leave the
 * browser, and for whom, is decided there (007).
 */
fun interface OutsideOpener {
    fun open(url: String): Boolean

    companion object {
        /**
         * Opens nothing, and says it did. Fake mode's: a test suite, or a screenshot run, must
         * not start the mail app of the device it runs on.
         */
        val nothing = OutsideOpener { true }
    }
}

/** The system's own: an activity that says it views the address, in a task of its own. */
class SystemOpener(private val context: Context) : OutsideOpener {
    override fun open(url: String): Boolean {
        val address = Uri.parse(url)
        if (address.scheme.isNullOrEmpty()) return false
        // Only an activity that offers itself to links from the web is started: the address
        // came from a page.
        val intent =
            Intent(Intent.ACTION_VIEW, address)
                .addCategory(Intent.CATEGORY_BROWSABLE)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        return try {
            context.startActivity(intent)
            true
        } catch (_: ActivityNotFoundException) {
            false
        } catch (_: SecurityException) {
            // An activity that takes it, and may not be started from here: as good as none.
            false
        }
    }
}

/** What came of asking the system to open a file (030). */
enum class FileOpened {
    /** The system took it. */
    TAKEN,

    /** Nothing on the device opens files of its kind. */
    NO_HANDLER,

    /** The file is not where it was saved. */
    MISSING,
}

/**
 * Opens a file that was downloaded, by the name it was saved under, with whatever the system
 * has for its kind (030).
 *
 * The shell opens only what the core asks it to (`OpenFile`): which downloads may be opened is
 * decided there, on a person's press in the list.
 */
fun interface FileOpener {
    fun open(fileName: String, mimeType: String): FileOpened

    companion object {
        /** Opens nothing, and says it did. Fake mode's: a test must not start another app. */
        val nothing = FileOpener { _, _ -> FileOpened.TAKEN }
    }
}

/**
 * The system's own, for a file in the device's Downloads: found among the files the app itself
 * saved there, by its name, and handed to an activity that views it, in a task of its own.
 * What the system is handed is the file and leave to read it: nothing of where it came from.
 */
class SystemFileOpener(
    private val find: (name: String) -> Uri?,
    private val kindOf: (file: Uri) -> String?,
    private val start: (Intent) -> Unit,
) : FileOpener {
    constructor(context: Context) : this(
        find = { name -> downloadedFile(context.contentResolver, name) },
        kindOf = { file -> context.contentResolver.getType(file) },
        start = context::startActivity,
    )

    override fun open(fileName: String, mimeType: String): FileOpened {
        val file = find(fileName) ?: return FileOpened.MISSING
        return try {
            start(viewing(file, kindOf(file), mimeType))
            FileOpened.TAKEN
        } catch (_: ActivityNotFoundException) {
            FileOpened.NO_HANDLER
        } catch (_: SecurityException) {
            // An activity that takes it, and may not be started from here: as good as none.
            FileOpened.NO_HANDLER
        }
    }

    companion object {
        /**
         * What asks the system to show a file: the file, its kind, leave to read that one file,
         * and a task of its own. The kind is what the system makes of the file as it is saved;
         * where it makes nothing of it, what the site said; and then any.
         */
        fun viewing(file: Uri, systemsKind: String?, sitesKind: String): Intent =
            Intent(Intent.ACTION_VIEW)
                .setDataAndType(file, systemsKind?.takeIf { it.isNotBlank() } ?: sitesKind.takeIf { it.isNotBlank() } ?: "*/*")
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
}

/**
 * The file of that name among the device's Downloads, as the system's record of it: the
 * newest, when a name was used more than once. Without a permission the app is shown only the
 * files it saved itself, which is all it looks for. Null when there is none.
 */
internal fun downloadedFile(resolver: ContentResolver, name: String): Uri? =
    try {
        val downloads = MediaStore.Downloads.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        resolver
            .query(
                downloads,
                arrayOf(MediaStore.MediaColumns._ID),
                "${MediaStore.MediaColumns.DISPLAY_NAME} = ? AND ${MediaStore.MediaColumns.IS_PENDING} = 0",
                arrayOf(name),
                "${MediaStore.MediaColumns.DATE_ADDED} DESC",
            )?.use { found -> if (found.moveToFirst()) ContentUris.withAppendedId(downloads, found.getLong(0)) else null }
    } catch (_: RuntimeException) {
        // A device whose record of files cannot be asked: the file is as good as gone.
        null
    }

/** Answers the core's host requests: the System service here, the rest by the engine adapter. */
class ShellHost(private val engine: HostServices, private val opener: OutsideOpener, private val files: FileOpener = FileOpener.nothing) : HostServices {
    override suspend fun answer(request: HostRequest): HostResponse {
        request.open_outside?.let { open ->
            return if (opener.open(open.url)) HostResponse(ack = Ack()) else HostResponse(error = Error(code = "system.no_handler"))
        }
        request.open_file?.let { open ->
            return when (files.open(open.file_name, open.mime_type)) {
                FileOpened.TAKEN -> HostResponse(ack = Ack())
                FileOpened.NO_HANDLER -> HostResponse(error = Error(code = "system.no_handler"))
                FileOpened.MISSING -> HostResponse(error = Error(code = "download.missing"))
            }
        }
        return engine.answer(request)
    }
}
