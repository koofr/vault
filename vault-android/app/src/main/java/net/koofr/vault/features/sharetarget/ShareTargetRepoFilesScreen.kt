package net.koofr.vault.features.sharetarget

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CreateNewFolder
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFilesBrowserDirCreated
import net.koofr.vault.RepoFilesBrowserItem
import net.koofr.vault.RepoFilesBrowserOptions
import net.koofr.vault.composables.EmptyFolderView
import net.koofr.vault.composables.RefreshableList
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.relativetime.relativeTime
import net.koofr.vault.features.repofiles.RepoFileRow
import net.koofr.vault.utils.queryEscape
import javax.inject.Inject

@HiltViewModel
class ShareTargetRepoFilesViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    val repoId: String = savedStateHandle.get<String>("repoId")!!
    val encryptedPath: String = savedStateHandle.get<String>("path")!!

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
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareTargetRepoFilesScreen(
    shareTargetVm: ShareTargetViewModel,
    vm: ShareTargetRepoFilesViewModel = hiltViewModel(),
) {
    val coroutineScope = rememberCoroutineScope()
    val navController = LocalNavController.current

    val info = subscribe(
        { v, cb -> v.repoFilesBrowsersInfoSubscribe(browserId = vm.browserId, cb = cb) },
        { v, id -> v.repoFilesBrowsersInfoData(id = id) },
    )

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(info.value?.title ?: "")
        }, actions = {
            IconButton(onClick = {
                info.value?.repoId?.let { repoId ->
                    vm.mobileVault.repoFilesBrowsersCreateDir(
                        browserId = vm.browserId,
                        cb = object : RepoFilesBrowserDirCreated {
                            override fun onCreated(encryptedPath: String) {
                                coroutineScope.launch {
                                    navController.navigate(
                                        "shareTarget/repos/$repoId/files?path=${
                                            queryEscape(
                                                encryptedPath,
                                            )
                                        }",
                                    )
                                }
                            }
                        },
                    )
                }
            }) {
                Icon(Icons.Filled.CreateNewFolder, "New folder")
            }
        })
    }, bottomBar = {
        ShareTargetBottomBar(shareTargetVm, uploadEnabled = true, onUploadClick = {
            shareTargetVm.upload(vm.repoId, vm.encryptedPath)
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        info.value?.let { info ->
            RefreshableList(
                modifier = Modifier.padding(paddingValues),
                status = info.status,
                isEmpty = info.items.isEmpty(),
                onRefresh = {
                    vm.mobileVault.repoFilesBrowsersLoadFiles(browserId = vm.browserId)
                },
                empty = {
                    EmptyFolderView()
                },
            ) {
                items(info.items, key = { it.file.id }) { item ->
                    ShareTargetRepoFilesListRow(vm, item)
                }
            }
        }
    }
}

@Composable
fun ShareTargetRepoFilesListRow(
    vm: ShareTargetRepoFilesViewModel,
    item: RepoFilesBrowserItem,
) {
    val navController = LocalNavController.current

    val fileIconBitmap = vm.fileIconCache.getIcon(
        FileIconProps(
            size = FileIconSize.SM,
            attrs = item.file.fileIconAttrs,
        ),
    )

    val modifiedDisplay = item.file.modified?.let {
        relativeTime(vm.mobileVault, it)
    }

    RepoFileRow(
        item.file,
        fileIconBitmap,
        modifiedDisplay = modifiedDisplay,
        checkboxChecked = false,
        onClick = {
            item.file.let { file ->
                navController.navigate(
                    "shareTarget/repos/${file.repoId}/files?path=${
                        queryEscape(
                            file.encryptedPath,
                        )
                    }",
                )
            }
        },
    )
}
