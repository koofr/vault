package net.koofr.vault.features.shareactivity

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

    private var transfersIsActiveSubscriptionId: UInt? = null
    private var transfersWasActive = false
    private var transfersAborted = false

    init {
        transfersIsActiveSubscriptionId =
            mobileVault.transfersIsActiveSubscribe(object : SubscriptionCallback {
                override fun onChange() {
                    viewModelScope.launch {
                        transfersIsActiveSubscriptionId?.let {
                            handleTransfersIsActive(it)
                        }
                    }
                }
            })

        handleTransfersIsActive(transfersIsActiveSubscriptionId!!)
    }

    override fun onCleared() {
        transfersIsActiveSubscriptionId?.let {
            mobileVault.unsubscribe(it)

            transfersIsActiveSubscriptionId = null
        }

        viewModelStore.clear()
    }

    private fun handleTransfersIsActive(id: UInt) {
        mobileVault.transfersIsActiveData(id)?.let { isActive ->
            if (isActive && !transfersWasActive) {
                transfersWasActive = true
            }
            if (!isActive && transfersWasActive) {
                if (transfersAborted) {
                    done()
                } else {
                    state.value = ShareActivityState.Done
                }
            }
        }
    }

    fun initFiles(intent: Intent) {
        val files = uploadHelper.getSendIntentFiles(intent) { ex ->
            mobileVault.notificationsShow(ex.toString())
        }.map {
            val localFile =
                mobileVault.localFilesFileInfo(it.name, LocalFileType.FILE, it.size, null)

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
                onUpload = {
                    val vm = TransfersViewModel(
                        mobileVault = mobileVault,
                        fileIconCache = fileIconCache,
                    ).also {
                        it.onAbort = {
                            transfersAborted = true
                        }

                        viewModelStore.put(it.javaClass.name, it)
                    }

                    state.value = ShareActivityState.Transfers(vm)
                },
                onCancel = {
                    cancel()
                },
            ).also {
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
