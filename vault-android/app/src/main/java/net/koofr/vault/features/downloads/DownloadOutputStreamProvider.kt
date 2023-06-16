package net.koofr.vault.features.downloads

import android.annotation.TargetApi
import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Environment
import android.os.ParcelFileDescriptor
import android.provider.MediaStore
import net.koofr.vault.DownloadStream
import net.koofr.vault.DownloadStreamProvider
import net.koofr.vault.R
import net.koofr.vault.SizeInfo
import net.koofr.vault.StreamException
import java.io.File

@TargetApi(29)
class DownloadOutputStreamProvider constructor(
    private val appContext: Context,
    private val onOpen: (Uri, String?) -> Unit,
) : DownloadStreamProvider {
    private var onDone: ((String?) -> Unit)? = null
    private var downloaded: Pair<Uri, String?>? = null

    override fun isRetriable(): Boolean {
        return true
    }

    override fun isOpenable(): Boolean {
        return true
    }

    override fun stream(
        name: String,
        size: SizeInfo,
        contentType: String?,
        uniqueName: String?,
    ): DownloadStream {
        try {
            this.downloaded = null

            val appName = appContext.resources.getString(R.string.app_name)
            val contentResolver = appContext.contentResolver

            val v = ContentValues()
            v.put(MediaStore.DownloadColumns.DISPLAY_NAME, name)
            v.put(
                MediaStore.DownloadColumns.RELATIVE_PATH,
                Environment.DIRECTORY_DOWNLOADS + File.separator + appName,
            )
            v.put(MediaStore.DownloadColumns.IS_PENDING, 1)
            when (size) {
                is SizeInfo.Exact -> {
                    v.put(MediaStore.DownloadColumns.SIZE, size.size)
                }

                else -> {}
            }
            contentType?.let {
                v.put(MediaStore.DownloadColumns.MIME_TYPE, it)
            }
            v.put(MediaStore.DownloadColumns.OWNER_PACKAGE_NAME, appContext.packageName)

            val insertUri =
                checkNotNull(contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, v)) {
                    "Failed to create download"
                }

            val fd = contentResolver.openFileDescriptor(insertUri, "w")

            val outputStream = ParcelFileDescriptor.AutoCloseOutputStream(fd)

            onDone = { error ->
                if (error != null) {
                    contentResolver.delete(insertUri, null, null)
                } else {
                    v.clear()
                    v.put(MediaStore.DownloadColumns.IS_PENDING, 0)
                    contentResolver.update(insertUri, v, null, null)

                    this.downloaded = Pair(insertUri, contentType)
                }
            }

            return DownloadOutpuStream(outputStream)
        } catch (ex: Exception) {
            onDone = null

            throw StreamException.IoException(ex.toString())
        }
    }

    override fun done(error: String?) {
        onDone?.let {
            it(error)

            onDone = null
        }
    }

    override fun open() {
        downloaded?.let { (uri, contentType) ->
            onOpen(uri, contentType)
        }
    }

    override fun dispose() {
        // nothing to do here
    }
}
