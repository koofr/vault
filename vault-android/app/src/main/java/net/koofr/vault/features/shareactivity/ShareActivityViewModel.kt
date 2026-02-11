package net.koofr.vault.features.shareactivity

import android.annotation.SuppressLint
import android.content.Intent
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.LocalFileType
import net.koofr.vault.MobileVault
import net.koofr.vault.SubscriptionCallback
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.sharetarget.ShareTargetFile
import net.koofr.vault.features.sharetarget.ShareTargetViewModel
import net.koofr.vault.features.transfers.TransfersViewModel
import net.koofr.vault.features.uploads.UploadHelper
import javax.inject.Inject

sealed class ShareActivityState {
    data object PreparingFiles : ShareActivityState()
    data object NoFiles : ShareActivityState()
    data class ShareTarget(val vm: ShareTargetViewModel) : ShareActivityState()
    data class Transfers(val vm: TransfersViewModel) : ShareActivityState()
    data object Done : ShareActivityState()
}

@HiltViewModel
class ShareActivityViewModel @Inject constructor(
    val mobileVault: MobileVault,
    private val uploadHelper: UploadHelper,
    val fileIconCache: FileIconCache,
) : ViewModel() {
    private val viewModelStore = ViewModelStore()

    val state = mutableStateOf<ShareActivityState>(ShareActivityState.PreparingFiles)

    var onCancel: (() -> Unit)? = null
    var onDone: (() -> Unit)? = null

    private var transfersDoneSessionsCountSubscriptionId: UInt? = null
    private var transfersDoneSessionsCountBeforeUpload: UInt? = null
    private var transfersAborted = false

    init {
        val id = mobileVault.transfersDoneSessionsCountSubscribe(
            cb = object : SubscriptionCallback {
                override fun onChange(id: UInt) {
                    viewModelScope.launch {
                        handleTransfersDoneSessionsCount(id)
                    }
                }
            },
        )

        transfersDoneSessionsCountSubscriptionId = id

        handleTransfersDoneSessionsCount(id)
    }

    override fun onCleared() {
        transfersDoneSessionsCountSubscriptionId?.let {
            mobileVault.unsubscribe(id = it)

            transfersDoneSessionsCountSubscriptionId = null
        }

        viewModelStore.clear()
    }

    private fun handleTransfersDoneSessionsCount(id: UInt) {
        mobileVault.transfersDoneSessionsCountData(id = id)?.let { transfersDoneSessionsCount ->
            transfersDoneSessionsCountBeforeUpload?.let { transfersDoneSessionsCountBeforeUpload ->
                // If there are more done sessions then before the upload,
                // transfers are done.
                if (transfersDoneSessionsCount > transfersDoneSessionsCountBeforeUpload) {
                    if (transfersAborted) {
                        done()
                    } else {
                        state.value = ShareActivityState.Done
                    }
                }
            }
        }
    }

    fun initFiles(intent: Intent) {
        val files = uploadHelper.getSendIntentFiles(intent) { ex ->
            mobileVault.notificationsShow(message = ex.toString())
        }.map {
            val localFile =
                mobileVault.localFilesFileInfo(name = it.name, typ = LocalFileType.FILE, size = it.size, modified = null)

            ShareTargetFile(localFile = localFile, uploadFile = it)
        }

        if (files.isEmpty()) {
            state.value = ShareActivityState.NoFiles
        } else {
            val vm = ShareTargetViewModel(
                mobileVault = mobileVault,
                uploadHelper = uploadHelper,
                fileIconCache = fileIconCache,
                files = files,
                beforeUpload = {
                    transfersDoneSessionsCountSubscriptionId?.let { id ->
                        transfersDoneSessionsCountBeforeUpload = mobileVault.transfersDoneSessionsCountData(id = id)
                    }
                },
                onUpload = {
                    val vm = TransfersViewModel(
                        mobileVault = mobileVault,
                        fileIconCache = fileIconCache,
                    ).also {
                        it.onAbort = {
                            transfersAborted = true
                        }

                        @SuppressLint("RestrictedApi")
                        viewModelStore.put(it.javaClass.name, it)
                    }

                    state.value = ShareActivityState.Transfers(vm)
                },
                onCancel = {
                    cancel()
                },
            ).also {
                @SuppressLint("RestrictedApi")
                viewModelStore.put(it.javaClass.name, it)
            }

            state.value = ShareActivityState.ShareTarget(vm)
        }
    }

    fun cancel() {
        onCancel?.let {
            onCancel = null
            onDone = null

            it()
        }
    }

    fun done() {
        onDone?.let {
            onCancel = null
            onDone = null

            it()
        }
    }
}
