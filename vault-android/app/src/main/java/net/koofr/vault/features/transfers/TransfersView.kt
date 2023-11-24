package net.koofr.vault.features.transfers

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.Transfer
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.subscribe
import javax.inject.Inject

@HiltViewModel
class TransfersViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
) : ViewModel() {
    var onAbort: (() -> Unit)? = null
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TransfersView(
    vm: TransfersViewModel = hiltViewModel(),
) {
    val transfersSummary = subscribe(
        { v, cb -> v.transfersSummarySubscribe(cb = cb) },
        { v, id -> v.transfersSummaryData(id = id) },
    )
    val transfersList = subscribe(
        { v, cb -> v.transfersListSubscribe(cb = cb) },
        { v, id -> v.transfersListData(id = id) },
    )

    Scaffold(topBar = {
        TopAppBar(title = {
            Text("Transfers")
        }, actions = {
            transfersSummary.value?.let { summary ->
                if (summary.canRetryAll) {
                    TextButton(onClick = {
                        vm.mobileVault.transfersRetryAll()
                    }) {
                        Text("RETRY ALL")
                    }
                }

                if (summary.canAbortAll) {
                    if (summary.isAllDone) {
                        TextButton(onClick = {
                            vm.onAbort?.invoke()

                            vm.mobileVault.transfersAbortAll()
                        }) {
                            Text("CLEAR")
                        }
                    } else {
                        TextButton(onClick = {
                            vm.onAbort?.invoke()

                            vm.mobileVault.transfersAbortAll()
                        }) {
                            Text("CANCEL ALL", color = MaterialTheme.colorScheme.error)
                        }
                    }
                }
            }
        })
    }, bottomBar = {
        transfersSummary.value?.let {
            TransfersSummaryBottomBar(it)
        }
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        transfersList.value?.let { transfers ->
            LazyColumn(modifier = Modifier.padding(paddingValues)) {
                items(transfers, { transfer -> transfer.id.toString() }) { transfer ->
                    TransfersViewRow(vm, transfer)
                }
            }
        }
    }
}

@Composable
fun TransfersViewRow(
    vm: TransfersViewModel,
    transfer: Transfer,
) {
    val fileIconBitmap = vm.fileIconCache.getIcon(
        FileIconProps(
            size = FileIconSize.SM,
            attrs = transfer.fileIconAttrs,
        ),
    )

    TransferRow(transfer, fileIconBitmap = fileIconBitmap, onRetry = {
        vm.mobileVault.transfersRetry(id = transfer.id)
    }, onAbort = {
        vm.onAbort?.invoke()

        vm.mobileVault.transfersAbort(id = transfer.id)
    }, onOpen = {
        vm.mobileVault.transfersOpen(id = transfer.id)
    })
}
