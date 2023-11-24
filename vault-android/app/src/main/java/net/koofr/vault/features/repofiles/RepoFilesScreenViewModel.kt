package net.koofr.vault.features.repofiles

import android.content.Intent
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SheetState
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavController
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesBrowserOptions
import net.koofr.vault.features.downloads.DownloadHelper
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.uploads.UploadHelper
import javax.inject.Inject

@OptIn(ExperimentalMaterial3Api::class)
@HiltViewModel
class RepoFilesScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    private val uploadHelper: UploadHelper,
    private val downloadHelper: DownloadHelper,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!
    private val encryptedPath: String = savedStateHandle.get<String>("path")!!

    val menuExpanded = mutableStateOf(false)

    val fileInfoSheetState = mutableStateOf(SheetState(true, SheetValue.Hidden, { true }, false))
    val fileInfoSheetFile = mutableStateOf<RepoFile?>(null)

    val sortSheetVisible = mutableStateOf(false)
    val sortSheetState = mutableStateOf(SheetState(false, SheetValue.Hidden, { true }, false))

    val browserId = mobileVault.repoFilesBrowsersCreate(
        repoId = repoId,
        encryptedPath = encryptedPath,
        options = RepoFilesBrowserOptions(
            selectName = null,
        ),
    )

    override fun onCleared() {
        super.onCleared()

        mobileVault.repoFilesBrowsersDestroy(browserId = browserId)
    }

    fun uploadFile(intent: Intent) {
        viewModelScope.launch(Dispatchers.IO) {
            val files = uploadHelper.getGetContentIntentFiles(intent) { ex ->
                mobileVault.notificationsShow(message = ex.toString())
            }

            uploadHelper.uploadFiles(repoId, encryptedPath, files)
        }
    }

    fun downloadFile(navController: NavController, repoFile: RepoFile) {
        downloadHelper.downloadRepoFile(navController, repoFile)
    }

    fun downloadSelected(navController: NavController) {
        downloadHelper.downloadRepoFilesBrowsersSelected(navController, browserId)

        mobileVault.repoFilesBrowsersClearSelection(browserId = browserId)
    }
}
