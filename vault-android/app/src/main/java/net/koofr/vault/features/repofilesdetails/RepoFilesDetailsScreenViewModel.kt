package net.koofr.vault.features.repofilesdetails

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import androidx.compose.runtime.mutableStateOf
import androidx.core.content.FileProvider
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavController
import coil.ImageLoader
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.launch
import net.koofr.vault.BuildConfig
import net.koofr.vault.FileCategory
import net.koofr.vault.FilesFilter
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesDetailsOptions
import net.koofr.vault.TransfersDownloadDone
import net.koofr.vault.features.downloads.DownloadHelper
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.Subscription
import net.koofr.vault.features.repo.RepoGuardViewModel
import net.koofr.vault.features.repo.WithRepoGuardViewModel
import net.koofr.vault.features.storage.StorageHelper
import java.io.File
import javax.inject.Inject

@HiltViewModel
class RepoFilesDetailsScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    private val storageHelper: StorageHelper,
    private val downloadHelper: DownloadHelper,
    private val imageLoader: ImageLoader,
    savedStateHandle: SavedStateHandle,
    @SuppressLint("StaticFieldLeak") @ApplicationContext private val appContext: Context,
) : ViewModel(), WithRepoGuardViewModel {
    private var repoGuardViewModel: RepoGuardViewModel? = null

    private val repoId: String = savedStateHandle.get<String>("repoId")!!
    private val encryptedPath: String = savedStateHandle.get<String>("path")!!

    val detailsId = mobileVault.repoFilesDetailsCreate(
        repoId = repoId,
        encryptedPath = encryptedPath,
        isEditing = false,
        options = RepoFilesDetailsOptions(
            loadContent = FilesFilter(
                categories = listOf(FileCategory.CODE, FileCategory.TEXT),
                exts = emptyList(),
            ),
            autosaveIntervalMs = 20000u,
        ),
    ).also {
        addCloseable {
            mobileVault.repoFilesDetailsDestroy(detailsId = it)
        }
    }

    val info = Subscription(
        mobileVault = mobileVault,
        coroutineScope = viewModelScope,
        subscribe = { v, cb -> v.repoFilesDetailsInfoSubscribe(detailsId = detailsId, cb = cb) },
        getData = { v, id ->
            v.repoFilesDetailsInfoData(id = id).also {
                it?.let {
                    repoGuardViewModel?.update(it.repoStatus, it.isLocked)
                }
            }
        },
    ).also {
        addCloseable(it)
    }

    var currentFile: RepoFile? = null

    val content =
        mutableStateOf<RepoFilesDetailsScreenContent>(RepoFilesDetailsScreenContent.Loading)

    override fun onCleared() {
        super.onCleared()

        content.value.close()
    }

    override fun setRepoGuardViewModel(repoGuardViewModel: RepoGuardViewModel) {
        if (this.repoGuardViewModel != null) {
            return
        }

        this.repoGuardViewModel = repoGuardViewModel

        addCloseable {
            this.repoGuardViewModel = null
        }

        info.data.value?.let {
            repoGuardViewModel.update(it.repoStatus, it.isLocked)
        }
    }

    fun setContent(newContent: RepoFilesDetailsScreenContent) {
        content.value.close()

        content.value = newContent
    }

    fun load(file: RepoFile) {
        currentFile = file

        val loader = RepoFilesDetailsScreenContentData.getLoader(appContext, file, imageLoader)

        if (loader != null) {
            setContent(RepoFilesDetailsScreenContent.Downloading)

            mobileVault.repoFilesDetailsDownloadTempFile(
                detailsId = detailsId,
                localBasePath = storageHelper.getTempDir(),
                onDone = object : TransfersDownloadDone {
                    override fun onDone(localFilePath: String, contentType: String?) {
                        viewModelScope.launch {
                            val localFile = File(localFilePath)

                            // prevent loading race conditions
                            if (currentFile == file) {
                                val data = loader(localFile)

                                setContent(
                                    RepoFilesDetailsScreenContent.Downloaded(
                                        file,
                                        localFile,
                                        data,
                                    ),
                                )
                            }
                        }
                    }
                },
            )
        } else {
            setContent(RepoFilesDetailsScreenContent.NotSupported(file))
        }
    }

    fun share(context: Context, localFile: File, contentType: String?) {
        val uri = FileProvider.getUriForFile(context, BuildConfig.FILES_AUTHORITY, localFile)

        val intent = Intent().apply {
            action = Intent.ACTION_SEND
            putExtra(Intent.EXTRA_STREAM, uri)
            contentType?.let {
                type = it
            }
        }

        context.startActivity(Intent.createChooser(intent, null))
    }

    fun download(navController: NavController, file: RepoFile) {
        downloadHelper.downloadRepoFile(navController, file)
    }

    fun rename() {
        info.data.value?.let { info ->
            info.repoId?.let { repoId ->
                info.encryptedPath?.let { encryptedPath ->
                    mobileVault.repoFilesRenameFile(repoId = repoId, encryptedPath = encryptedPath)
                }
            }
        }
    }

    fun delete() {
        mobileVault.repoFilesDetailsDelete(detailsId = detailsId)
    }
}
