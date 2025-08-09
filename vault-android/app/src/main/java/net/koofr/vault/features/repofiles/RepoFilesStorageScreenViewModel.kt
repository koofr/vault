package net.koofr.vault.features.repofiles

import androidx.lifecycle.SavedStateHandle
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFilesBrowserSource
import net.koofr.vault.features.downloads.DownloadHelper
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.uploads.UploadHelper
import javax.inject.Inject

@HiltViewModel
class RepoFilesStorageScreenViewModel @Inject constructor(
    mobileVault: MobileVault,
    fileIconCache: FileIconCache,
    uploadHelper: UploadHelper,
    downloadHelper: DownloadHelper,
    savedStateHandle: SavedStateHandle,
) : RepoFilesScreenViewModel(
    mobileVault = mobileVault,
    fileIconCache = fileIconCache,
    uploadHelper = uploadHelper,
    downloadHelper = downloadHelper,
    source = RepoFilesBrowserSource.Storage(
        repoId = savedStateHandle.get<String>("repoId")!!,
        encryptedPath = savedStateHandle.get<String>("path")!!,
    ),
)
