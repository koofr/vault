package net.koofr.vault.features.uploads

import android.annotation.SuppressLint
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import androidx.core.database.getLongOrNull
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.MobileVault
import net.koofr.vault.utils.slugify
import java.io.FileNotFoundException
import java.util.UUID

class UploadHelper(private val mobileVault: MobileVault, private val appContext: Context) {
    fun uploadFiles(repoId: String, encryptedParentPath: String, files: List<UploadFile>) {
        files.forEach { file ->
            file.data.let { data ->
                when (data) {
                    is UploadFileData.Stream -> {
                        mobileVault.transfersUploadStream(
                            repoId = repoId,
                            encryptedParentPath = encryptedParentPath,
                            name = file.name,
                            streamProvider = UploadInputStreamProvider(data.stream, file.size),
                        )
                    }

                    is UploadFileData.Bytes -> {
                        mobileVault.transfersUploadBytes(
                            repoId = repoId,
                            encryptedParentPath = encryptedParentPath,
                            name = file.name,
                            bytes = data.bytes,
                        )
                    }
                }
            }
        }
    }

    fun getGetContentIntentFiles(intent: Intent, onError: (Exception) -> Unit): List<UploadFile> {
        val files = mutableListOf<UploadFile>()

        val contentResolver = appContext.contentResolver

        intent.clipData?.let { clipData ->
            for (i in 0 until clipData.itemCount) {
                val item = clipData.getItemAt(i)

                handleUri(contentResolver, files, onError, item.uri)
            }
        }

        intent.data?.let {
            handleUri(contentResolver, files, onError, it)
        }

        return files
    }

    fun getSendIntentFiles(intent: Intent, onError: (Exception) -> Unit): List<UploadFile> {
        val files = mutableListOf<UploadFile>()

        val contentResolver = appContext.contentResolver

        intent.extras?.let { extras ->
            when (intent.action) {
                Intent.ACTION_SEND -> {
                    if (extras.containsKey(Intent.EXTRA_TEXT)) {
                        try {
                            files.add(textIntentToUploadFile(extras))
                        } catch (ex: Exception) {
                            onError(ex)
                        }
                    }
                    if (extras.containsKey(Intent.EXTRA_STREAM)) {
                        try {
                            files.add(streamIntentToUploadFile(contentResolver, extras))
                        } catch (ex: Exception) {
                            onError(ex)
                        }
                    }
                }

                Intent.ACTION_SEND_MULTIPLE -> {
                    if (extras.containsKey(Intent.EXTRA_STREAM)) {
                        @Suppress("DEPRECATION")
                        extras.getParcelableArrayList<Uri>(Intent.EXTRA_STREAM)
                            ?.let {
                                it.forEach { uri ->
                                    handleUri(contentResolver, files, onError, uri)
                                }
                            }
                    }
                }
            }
        }

        return files
    }

    private fun handleUri(
        contentResolver: ContentResolver,
        files: MutableList<UploadFile>,
        onError: (Exception) -> Unit,
        uri: Uri,
    ) {
        if (isUriDocumentsTree(uri)) {
            handleUriDocumentsRoot(contentResolver, files, onError, uri)
        } else {
            try {
                files.add(uriToUploadFile(contentResolver, uri))
            } catch (ex: Exception) {
                onError(ex)
            }
        }
    }

    @SuppressLint("Range")
    private fun handleUriDocumentsRoot(
        contentResolver: ContentResolver,
        files: MutableList<UploadFile>,
        onError: (Exception) -> Unit,
        uri: Uri,
    ) {
        try {
            val name = queryName(
                contentResolver,
                DocumentsContract.buildDocumentUriUsingTree(
                    uri,
                    DocumentsContract.getTreeDocumentId(uri),
                ),
            )

            val childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(
                uri,
                DocumentsContract.getTreeDocumentId(uri),
            )

            handleUriDocumentsDir(contentResolver, files, onError, uri, childrenUri, "$name/")
        } catch (ex: Exception) {
            onError(ex)
        }
    }

    @SuppressLint("Range")
    private fun handleUriDocumentsDir(
        contentResolver: ContentResolver,
        files: MutableList<UploadFile>,
        onError: (Exception) -> Unit,
        treeUri: Uri,
        uri: Uri,
        namePrefix: String,
    ) {
        val items = queryItems(contentResolver, treeUri, uri).sortedBy { it.name }

        for (item in items) {
            try {
                if (item.type is DocumentItemType.Dir) {
                    handleUriDocumentsDir(
                        contentResolver,
                        files,
                        onError,
                        treeUri,
                        item.uri,
                        "${namePrefix}${item.name}/",
                    )
                } else {
                    val stream = contentResolver.openInputStream(item.uri)
                        ?: throw FileNotFoundException("Failed to open resource input stream: $uri")

                    files.add(
                        UploadFile(
                            name = "${namePrefix}${item.name}",
                            size = item.size,
                            data = UploadFileData.Stream(stream),
                        ),
                    )
                }
            } catch (ex: Exception) {
                onError(ex)
            }
        }
    }

    @SuppressLint("Range")
    private fun queryName(contentResolver: ContentResolver, uri: Uri): String {
        return checkNotNull(contentResolver.query(uri, null, null, null, null)) {
            "contentResolver query cursor is null"
        }.use { cursor ->
            if (cursor.moveToFirst()) {
                cursor.getString(cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME))
            } else {
                throw IllegalStateException("Name not found")
            }
        }
    }

    sealed class DocumentItemType {
        data object File : DocumentItemType()
        data object Dir : DocumentItemType()
    }

    data class DocumentItem(
        val name: String,
        val size: Long?,
        val uri: Uri,
        val type: DocumentItemType,
    )

    @SuppressLint("Range")
    private fun queryItems(
        contentResolver: ContentResolver,
        treeUri: Uri,
        uri: Uri,
    ): List<DocumentItem> {
        val items = mutableListOf<DocumentItem>()

        contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val idColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val nameColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val mimeColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val sizeColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_SIZE)

            while (cursor.moveToNext()) {
                val id = cursor.getString(idColumn)
                val name = cursor.getString(nameColumn)
                val mime = cursor.getString(mimeColumn)

                if (mime == DocumentsContract.Document.MIME_TYPE_DIR) {
                    items.add(
                        DocumentItem(
                            name = name,
                            size = null,
                            uri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, id),
                            type = DocumentItemType.Dir,
                        ),
                    )
                } else {
                    items.add(
                        DocumentItem(
                            name = name,
                            size = cursor.getLongOrNull(sizeColumn),
                            uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, id),
                            type = DocumentItemType.File,
                        ),
                    )
                }
            }
        }

        return items
    }

    private fun isUriDocumentsTree(uri: Uri): Boolean {
        if (uri.scheme != ContentResolver.SCHEME_CONTENT) {
            return false
        }

        val paths = uri.pathSegments

        return paths.size >= 2 && paths[0] == "tree"
    }

    private fun textIntentToUploadFile(extras: Bundle): UploadFile {
        val name = extras.getString(Intent.EXTRA_SUBJECT)

        val data =
            checkNotNull(extras.getCharSequence(Intent.EXTRA_TEXT)?.toString()) {
                "Missing intent text"
            }

        return textToUploadFile(name, data)
    }

    private fun streamIntentToUploadFile(
        contentResolver: ContentResolver,
        extras: Bundle,
    ): UploadFile {
        @Suppress("DEPRECATION")
        val uri =
            checkNotNull(extras.getParcelable<Uri>(Intent.EXTRA_STREAM)) {
                "Missing intent stream uri"
            }

        return uriToUploadFile(contentResolver, uri)
    }

    private fun textToUploadFile(initialName: String?, data: String): UploadFile {
        var name = initialName

        if (name != null) {
            name = slugify(name)
        }

        if (name.isNullOrEmpty()) {
            name = getRandomName("text", "txt")
        }

        val dataBytes = data.toByteArray(Charsets.UTF_8)

        return UploadFile(
            name = name,
            size = dataBytes.size.toLong(),
            data = UploadFileData.Bytes(dataBytes),
        )
    }

    private fun uriToUploadFile(contentResolver: ContentResolver, uri: Uri): UploadFile {
        var (name, size) = uriToNameSize(contentResolver, uri)

        if (name == null) {
            name = uri.lastPathSegment
        }

        if (name == null) {
            name = getRandomName("upload", null)
        }

        val stream = contentResolver.openInputStream(uri)
            ?: throw FileNotFoundException("Failed to open resource input stream: $uri")

        return UploadFile(name = name, size = size, data = UploadFileData.Stream(stream))
    }

    private fun uriToNameSize(resolver: ContentResolver, uri: Uri): Pair<String?, Long?> {
        var name: String? = null
        var size: Long? = null

        resolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE),
            null,
            null,
            null,
        )?.use { cursor ->
            if (cursor.moveToFirst()) {
                name = cursor.getString(0)

                size = resolver.openFileDescriptor(uri, "r")?.use {
                    it.statSize
                }
            }
        }

        return Pair(name, size)
    }

    private fun getRandomName(base: String, suffix: String?): String {
        var name = base + "-" + UUID.randomUUID().toString().substring(0, 8)

        suffix?.let {
            name = "$name.$it"
        }

        return name
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object UploadHelperModule {
    @ActivityRetainedScoped
    @Provides
    fun provideUploadHelper(
        mobileVault: MobileVault,
        @ApplicationContext appContext: Context,
    ): UploadHelper {
        return UploadHelper(mobileVault, appContext)
    }
}
