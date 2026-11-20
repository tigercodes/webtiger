// The real fetcher (013 §6): the system's DownloadManager. It fetches an address again, in a
// process of its own, with the cookies the adapter hands it: so it gets what an address alone
// can get, and not what only the page could (a `blob:`, the answer to a posted form).
package com.tigercodes.webtiger.webview

import android.app.DownloadManager
import android.content.Context
import android.net.Uri
import android.os.Environment
import android.webkit.CookieManager
import androidx.webkit.ProfileStore
import androidx.webkit.WebViewFeature

/** Where downloads are saved. */
enum class DownloadPlace {
    /** The device's Downloads, which every app and the person see. Only the app itself, started for real, saves there. */
    DEVICE,

    /** The app's own folder for downloads: what a test saves into, and fake mode. */
    APP,
}

internal class SystemFetcher(private val context: Context, private val place: DownloadPlace) : Fetcher {
    private val manager: DownloadManager? = context.getSystemService(DownloadManager::class.java)

    override fun start(offer: DownloadOffer, loginSet: String, name: String): Long? =
        try {
            val request = DownloadManager.Request(Uri.parse(offer.url)).setTitle(name)
            if (offer.mimeType.isNotEmpty()) request.setMimeType(offer.mimeType)
            if (offer.userAgent.isNotEmpty()) request.addRequestHeader("User-Agent", offer.userAgent)
            cookies(offer.url, loginSet)?.let { request.addRequestHeader("Cookie", it) }
            // The name is the core's and has no folder in it; it is held to that here all the same.
            val file = name.substringAfterLast('/').substringAfterLast('\\').ifEmpty { "download" }
            when (place) {
                DownloadPlace.DEVICE -> {
                    request.setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, file)
                    // The system's own notice of a download stays when the file is whole: it is
                    // how a person opens it, until the list can (013 §2).
                    request.setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                }
                DownloadPlace.APP -> request.setDestinationInExternalFilesDir(context, Environment.DIRECTORY_DOWNLOADS, file)
            }
            manager?.enqueue(request)
        } catch (_: RuntimeException) {
            // An address the system will not fetch, or a place it cannot write to.
            null
        }

    override fun stop(handle: Long) {
        manager?.remove(handle)
    }

    override fun status(handle: Long): FetchStatus? {
        val cursor = manager?.query(DownloadManager.Query().setFilterById(handle)) ?: return null
        return cursor.use {
            if (!it.moveToFirst()) return null
            fun long(column: String): Long = it.getLong(it.getColumnIndexOrThrow(column))
            when (it.getInt(it.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))) {
                DownloadManager.STATUS_SUCCESSFUL -> {
                    // Where the system saved it says what it called it: the name asked for, or
                    // another when that was taken. Where it does not say a file, the name asked
                    // for is all that is known: it is the fetch's title.
                    val saved = Uri.parse(it.getString(it.getColumnIndexOrThrow(DownloadManager.COLUMN_LOCAL_URI)).orEmpty())
                    val name = if (saved.scheme == "file") saved.lastPathSegment.orEmpty().substringAfterLast('/') else ""
                    FetchStatus.Finished(name.ifEmpty { it.getString(it.getColumnIndexOrThrow(DownloadManager.COLUMN_TITLE)).orEmpty() })
                }
                DownloadManager.STATUS_FAILED -> FetchStatus.Failed(code(it.getInt(it.getColumnIndexOrThrow(DownloadManager.COLUMN_REASON))))
                else -> FetchStatus.Coming(long(DownloadManager.COLUMN_BYTES_DOWNLOADED_SO_FAR), long(DownloadManager.COLUMN_TOTAL_SIZE_BYTES))
            }
        }
    }

    /** The login set's cookies for an address, as a request's header carries them. */
    private fun cookies(url: String, loginSet: String): String? =
        try {
            if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                ProfileStore.getInstance().getProfile(loginSet)?.cookieManager?.getCookie(url)
            } else {
                CookieManager.getInstance().getCookie(url)
            }
        } catch (_: RuntimeException) {
            null
        }

    /** The system's reason as one of the contract's codes. The reason's own text never leaves the shell. */
    private fun code(reason: Int): String =
        when (reason) {
            DownloadManager.ERROR_FILE_ERROR,
            DownloadManager.ERROR_INSUFFICIENT_SPACE,
            DownloadManager.ERROR_DEVICE_NOT_FOUND,
            DownloadManager.ERROR_FILE_ALREADY_EXISTS,
            -> "download.disk"
            DownloadManager.ERROR_CANNOT_RESUME,
            DownloadManager.ERROR_HTTP_DATA_ERROR,
            DownloadManager.ERROR_TOO_MANY_REDIRECTS,
            DownloadManager.ERROR_UNHANDLED_HTTP_CODE,
            -> "download.network"
            else -> "download.failed"
        }
}
