package net.koofr.vault.features.sharetarget

import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.uploads.UploadHelper

class ShareTargetViewModel constructor(
    val mobileVault: MobileVault,
    private val uploadHelper: UploadHelper,
    val fileIconCache: FileIconCache,
    val files: List<ShareTargetFile>,
    private val onUpload: () -> Unit,
    private val onCancel: () -> Unit,
) : ViewModel() {
    val filesDialogVisible = mutableStateOf(false)

    override fun onCleared() {
        super.onCleared()
    }

    fun cancel() {
        onCancel()
    }

    fun upload(repoId: String, encryptedPath: String) {
        uploadHelper.uploadFiles(repoId, encryptedPath, files.map { it.uploadFile })

        onUpload()
    }

    fun showFilesDialog() {
        filesDialogVisible.value = true
    }

    fun hideFilesDialog() {
        filesDialogVisible.value = false
    }
}
