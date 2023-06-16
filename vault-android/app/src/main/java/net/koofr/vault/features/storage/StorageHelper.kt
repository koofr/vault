package net.koofr.vault.features.storage

import android.content.Context
import android.net.Uri
import android.os.Environment
import androidx.core.content.FileProvider
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.BuildConfig
import net.koofr.vault.R
import java.io.File
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import javax.inject.Singleton

class StorageHelper constructor(private val appContext: Context) {
    private fun getCacheDir(): File {
        return appContext.externalCacheDir ?: appContext.cacheDir
    }

    fun getTempDir(): String {
        val dir = File(getCacheDir().absolutePath + File.separator + "files")

        if (!dir.exists()) {
            dir.mkdirs()
        }

        return dir.absolutePath
    }

    @Throws(IOException::class)
    fun getDownloadsDir(): String {
        val appName = appContext.resources.getString(R.string.app_name)

        val baseDir = appContext.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)
            ?: throw IOException("Downloads dir not found")

        val dir = File(baseDir.absolutePath + File.separator + appName)

        if (!dir.exists()) {
            dir.mkdirs()
        }

        return dir.absolutePath
    }

    @Throws(IOException::class)
    fun clearCache() {
        getCacheDir().let { cacheDir ->
            if (cacheDir.exists()) {
                cacheDir.listFiles()?.let {
                    for (child in it) {
                        child.deleteRecursively()
                    }
                }
            }
        }
    }

    @Throws(IOException::class)
    fun createTempFile(suffix: String? = null): File {
        val storageDir = File(getTempDir())

        return File.createTempFile("tmp", suffix, storageDir)
    }

    @Throws(IOException::class)
    fun createImageFile(): File {
        val timeStamp =
            SimpleDateFormat("yyyyMMdd-HHmmss", Locale.ROOT).format(Date())
        val imageFileName = "photo-$timeStamp"
        val storageDir = File(getTempDir())
        return File.createTempFile(imageFileName, ".jpg", storageDir)
    }

    @Throws(IOException::class)
    fun createImageFileUri(): Pair<File, Uri> {
        val file = createImageFile()

        val uri = FileProvider.getUriForFile(appContext, BuildConfig.FILES_AUTHORITY, file)

        return Pair(file, uri)
    }
}

@Module
@InstallIn(SingletonComponent::class)
object StorageHelperModule {
    @Singleton
    @Provides
    fun provideStorageHelper(@ApplicationContext appContext: Context): StorageHelper {
        return StorageHelper(appContext)
    }
}
