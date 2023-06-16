package net.koofr.vault.features.downloads

import android.app.DownloadManager
import android.app.Service
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.navigation.NavController
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.DownloadStreamProvider
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFile
import net.koofr.vault.TransfersDownloadDone
import net.koofr.vault.TransfersDownloadOpen
import net.koofr.vault.features.storage.StorageHelper
import net.koofr.vault.features.transfers.TransfersHelper
import java.io.File

class DownloadHelper(
    private val mobileVault: MobileVault,
    private val storageHelper: StorageHelper,
    private val transfersHelper: TransfersHelper,
    private val appContext: Context,
) {
    fun downloadRepoFile(navController: NavController, repoFile: RepoFile) {
        repoFile.path?.let { path ->
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                mobileVault.transfersDownloadStream(
                    repoId = repoFile.repoId,
                    path = path,
                    streamProvider = downloadStreamProvider(),
                )
            } else {
                val localFilePath = try {
                    storageHelper.getDownloadsDir()
                } catch (ex: Exception) {
                    mobileVault.notificationsShow(ex.toString())

                    return
                }

                val transfersDownloadFileHandler = transfersDownloadFileHandler()

                mobileVault.transfersDownloadFile(
                    repoId = repoFile.repoId,
                    path = path,
                    localFilePath = localFilePath,
                    appendName = true,
                    autorename = true,
                    onOpen = transfersDownloadFileHandler,
                    onDone = transfersDownloadFileHandler,
                )
            }
        }

        transfersHelper.navigateWhenActive(navController)
    }

    fun downloadRepoFilesBrowsersSelected(navController: NavController, browserId: UInt) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            mobileVault.repoFilesBrowsersDownloadSelectedStream(
                browserId,
                downloadStreamProvider(),
            )
        } else {
            val localFilePath = try {
                storageHelper.getDownloadsDir()
            } catch (ex: Exception) {
                mobileVault.notificationsShow(ex.toString())

                return
            }

            val transfersDownloadFileHandler = transfersDownloadFileHandler()

            mobileVault.repoFilesBrowsersDownloadSelectedFile(
                browserId = browserId,
                localFilePath = localFilePath,
                appendName = true,
                autorename = true,
                onOpen = transfersDownloadFileHandler,
                onDone = transfersDownloadFileHandler,
            )
        }

        transfersHelper.navigateWhenActive(navController)
    }

    private fun downloadStreamProvider(): DownloadStreamProvider {
        return DownloadOutputStreamProvider(appContext, onOpen = { uri, contentType ->
            open(uri, contentType)
        })
    }

    private fun transfersDownloadFileHandler(): TransfersDownloadHandler {
        return TransfersDownloadHandler(mobileVault, appContext, onOpen = ::open)
    }

    fun open(uri: Uri, contentType: String?) {
        try {
            val viewIntent = Intent(Intent.ACTION_VIEW)
            viewIntent.setDataAndType(uri, contentType)
            viewIntent.flags =
                Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK

            try {
                appContext.startActivity(viewIntent)
            } catch (ex: ActivityNotFoundException) {
                val viewDownloadsIntent = Intent(DownloadManager.ACTION_VIEW_DOWNLOADS)
                viewDownloadsIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK

                appContext.startActivity(viewDownloadsIntent)
            }
        } catch (ex: Exception) {
            mobileVault.notificationsShow(ex.toString())
        }
    }
}

class TransfersDownloadHandler(
    private val mobileVault: MobileVault,
    private val appContext: Context,
    private val onOpen: (Uri, String?) -> Unit,
) : TransfersDownloadDone, TransfersDownloadOpen {
    private var uri: Uri? = null

    override fun onDone(localFilePath: String, contentType: String?) {
        try {
            val localFile = File(localFilePath)

            val downloadManager =
                appContext.getSystemService(Service.DOWNLOAD_SERVICE) as DownloadManager

            @Suppress("DEPRECATION")
            val id = downloadManager.addCompletedDownload(
                localFile.name,
                "Koofr Vault download",
                false,
                contentType ?: "application/octet-stream",
                localFilePath,
                localFile.length(),
                true,
            )

            uri = downloadManager.getUriForDownloadedFile(id)
        } catch (ex: Exception) {
            mobileVault.notificationsShow(ex.toString())
        }
    }

    override fun onOpen(localFilePath: String, contentType: String?) {
        uri?.let {
            onOpen(it, contentType)
        }
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object DownloadHelperModule {
    @ActivityRetainedScoped
    @Provides
    fun provideDownloadHelper(
        mobileVault: MobileVault,
        storageHelper: StorageHelper,
        transfersHelper: TransfersHelper,
        @ApplicationContext appContext: Context,
    ): DownloadHelper {
        return DownloadHelper(mobileVault, storageHelper, transfersHelper, appContext)
    }
}
