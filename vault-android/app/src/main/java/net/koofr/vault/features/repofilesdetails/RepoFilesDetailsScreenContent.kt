package net.koofr.vault.features.repofilesdetails

import android.content.Context
import android.net.Uri
import androidx.media3.exoplayer.ExoPlayer
import coil.ImageLoader
import net.koofr.vault.FileCategory
import net.koofr.vault.RepoFile
import net.koofr.vault.composables.buildExoPlayer
import java.io.Closeable
import java.io.File

sealed class RepoFilesDetailsScreenContentData : Closeable {
    data class Image(
        val localFile: File,
        val ext: String,
        val imageLoader: ImageLoader,
    ) : RepoFilesDetailsScreenContentData() {
        companion object {
            val exts = setOf(
                "bmp",
                "cur",
                "gif",
                "heic",
                "ico",
                "jpeg",
                "jpg",
                "png",
                "svg",
                "webp",
            )
        }
    }

    data class Media(val exoPlayer: ExoPlayer) : RepoFilesDetailsScreenContentData() {
        override fun close() {
            exoPlayer.release()
        }

        companion object {
            val exts = setOf(
                "3gp",
                "aac",
                "amr",
                "flac",
                "imy",
                "m4a",
                "mid",
                "mkv",
                "mov",
                "mp3",
                "mp4",
                "mxmf",
                "ogg",
                "ota",
                "rtttl",
                "rtx",
                "wav",
                "webm",
                "xmf",
            )
        }
    }

    override fun close() {}

    companion object {
        fun isTextEditor(category: FileCategory?): Boolean {
            return category == FileCategory.TEXT || category == FileCategory.CODE
        }

        fun getLoader(
            context: Context,
            file: RepoFile,
            imageLoader: ImageLoader,
        ): ((File) -> RepoFilesDetailsScreenContentData)? {
            file.ext?.let { ext ->
                if (Image.exts.contains(ext)) {
                    return {
                        Image(it, ext, imageLoader)
                    }
                } else if (Media.exts.contains(ext)) {
                    return {
                        Media(buildExoPlayer(context, Uri.fromFile(it)))
                    }
                }
            }

            return null
        }
    }
}

sealed class RepoFilesDetailsScreenContent : Closeable {
    data object Loading : RepoFilesDetailsScreenContent()

    data object Downloading : RepoFilesDetailsScreenContent()

    data class Downloaded(
        val repoFile: RepoFile,
        val localFile: File,
        val data: RepoFilesDetailsScreenContentData,
    ) :
        RepoFilesDetailsScreenContent() {
        override fun close() {
            data.close()
        }
    }

    data object TextEditor : RepoFilesDetailsScreenContent()

    data class NotSupported(val file: RepoFile) : RepoFilesDetailsScreenContent()

    override fun close() {}
}
