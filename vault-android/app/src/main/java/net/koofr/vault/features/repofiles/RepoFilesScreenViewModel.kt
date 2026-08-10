package net.koofr.vault.features.repofiles

import android.content.Context
import android.content.Intent
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheetDefaults
import androidx.compose.material3.SheetState
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavController
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.koofr.vault.MobileVault
import net.koofr.vault.R
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesBrowserFileCreated
import net.koofr.vault.RepoFilesBrowserOptions
import net.koofr.vault.RepoFilesBrowserSource
import net.koofr.vault.features.downloads.DownloadHelper
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.Subscription
import net.koofr.vault.features.repo.RepoGuardViewModel
import net.koofr.vault.features.repo.WithRepoGuardViewModel
import net.koofr.vault.features.uploads.UploadHelper
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
open class RepoFilesScreenViewModel constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    private val uploadHelper: UploadHelper,
    private val downloadHelper: DownloadHelper,
    source: RepoFilesBrowserSource,
) : ViewModel(), WithRepoGuardViewModel {
    private var repoGuardViewModel: RepoGuardViewModel? = null

    val menuExpanded = mutableStateOf(false)

    val fileInfoSheetState = mutableStateOf(
        SheetState(
            skipPartiallyExpanded = true,
            velocityThreshold = { 0f },
            positionalThreshold = { 0f },
            initialValue = SheetValue.Hidden,
            confirmValueChange = { true },
            skipHiddenState = false
        )
    )
    val fileInfoSheetFile = mutableStateOf<RepoFile?>(null)

    val sortSheetVisible = mutableStateOf(false)
    val sortSheetState = mutableStateOf(
        SheetState(
            skipPartiallyExpanded = false,
            velocityThreshold = { 0f },
            positionalThreshold = { 0f },
            initialValue = SheetValue.Hidden,
            confirmValueChange = { true },
            skipHiddenState = false
        )
    )

    val browserId = mobileVault.repoFilesBrowsersCreate(
        source = source,
        options = RepoFilesBrowserOptions(
            selectName = null,
        ),
    ).also {
        addCloseable {
            mobileVault.repoFilesBrowsersDestroy(browserId = it)
        }
    }

    val info = Subscription(
        mobileVault = mobileVault,
        coroutineScope = viewModelScope,
        subscribe = { v, cb -> v.repoFilesBrowsersInfoSubscribe(browserId = browserId, cb = cb) },
        getData = { v, id ->
            v.repoFilesBrowsersInfoData(id = id).also {
                it?.let {
                    repoGuardViewModel?.update(it.repoStatus, it.isLocked)
                }
            }
        },
    ).also {
        addCloseable(it)
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

    fun uploadFile(intent: Intent) {
        info.data.value?.let { infoData ->
            infoData.repoId?.let { repoId ->
                infoData.encryptedPath?.let { encryptedPath ->
                    viewModelScope.launch(Dispatchers.IO) {
                        val files = uploadHelper.getGetContentIntentFiles(intent) { ex ->
                            mobileVault.notificationsShow(message = ex.toString())
                        }

                        uploadHelper.uploadFiles(repoId, encryptedPath, files)
                    }
                }
            }
        }
    }

    fun downloadFile(navController: NavController, repoFile: RepoFile) {
        downloadHelper.downloadRepoFile(navController, repoFile)
    }

    fun downloadSelected(navController: NavController) {
        downloadHelper.downloadRepoFilesBrowsersSelected(navController, browserId)

        mobileVault.repoFilesBrowsersClearSelection(browserId = browserId)
    }

    fun createTextFile(context: Context, onCreated: (String) -> Unit) {
        val dateFormat = SimpleDateFormat("yyyyMMddHHmmss", Locale.ROOT)
        val date = dateFormat.format(Date())
        val name = context.getString(R.string.repo_files_create_text_file_default_filename, date) + ".txt"

        mobileVault.repoFilesBrowsersCreateFile(
            browserId = browserId,
            name = name,
            cb = object : RepoFilesBrowserFileCreated {
                override fun onCreated(encryptedPath: String) {
                    viewModelScope.launch {
                        onCreated(encryptedPath)
                    }
                }
            },
        )
    }
}
