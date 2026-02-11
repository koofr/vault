package net.koofr.vault.features.repofiles

import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.R
import net.koofr.vault.RepoFilesBrowserDirCreated
import net.koofr.vault.composables.EmptyFolderView
import net.koofr.vault.composables.MultiAddButton
import net.koofr.vault.composables.MultiAddButtonItem
import net.koofr.vault.composables.RefreshableList
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.transfers.TransfersButton
import net.koofr.vault.features.uploads.takePicture
import net.koofr.vault.utils.CustomActivityResultContracts
import net.koofr.vault.utils.queryEscape

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesScreen(
    vm: RepoFilesScreenViewModel,
) {
    val context = LocalContext.current
    val navController = LocalNavController.current

    val moveInfo = subscribe(
        { v, cb -> v.repoFilesMoveInfoSubscribe(cb = cb) },
        { v, id -> v.repoFilesMoveInfoData(id = id) },
    )

    LaunchedEffect(moveInfo.value != null) {
        moveInfo.value?.let { moveInfo ->
            moveInfo.encryptedDestPathChain.forEach { encryptedDestPath ->
                navController.navigate(
                    "repos/${moveInfo.repoId}/files/move?path=${
                        queryEscape(
                            encryptedDestPath,
                        )
                    }",
                )
            }
        }
    }

    val takePicture = takePicture({ file ->
        vm.info.data.value?.let { info ->
            info.repoId?.let { repoId ->
                info.encryptedPath?.let { encryptedPath ->
                    vm.mobileVault.transfersUploadFile(
                        repoId = repoId,
                        encryptedParentPath = encryptedPath,
                        name = file.name,
                        localFilePath = file.absolutePath,
                        removeFileAfterUpload = true,
                    )
                }
            }
        }
    })

    val uploadFileLauncher =
        rememberLauncherForActivityResult(CustomActivityResultContracts.GetContent()) { intent ->
            intent?.let {
                vm.uploadFile(it)
            }
        }

    val uploadFile = remember {
        {
            uploadFileLauncher.launch(Unit)
        }
    }

    val uploadFolderLauncher =
        rememberLauncherForActivityResult(CustomActivityResultContracts.OpenDocumentTree()) { intent ->
            intent?.let {
                vm.uploadFile(it)
            }
        }

    val uploadFolder = remember {
        {
            uploadFolderLauncher.launch(Unit)
        }
    }

    val selectedCount = vm.info.data.value?.selectedCount ?: 0u
    val selectMode = selectedCount > 0u

    BackHandler(selectMode) {
        vm.mobileVault.repoFilesBrowsersClearSelection(browserId = vm.browserId)
    }

    Scaffold(topBar = {
        TopAppBar(
            title = {
                Text(
                    if (selectMode) {
                        pluralStringResource(
                            R.plurals.repo_files_edit_mode_items_count_label,
                            selectedCount.toInt(),
                            selectedCount.toInt(),
                        )
                    } else {
                        vm.info.data.value?.title ?: ""
                    },
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            },
            navigationIcon = {
                if (selectMode) {
                    IconButton(onClick = { vm.mobileVault.repoFilesBrowsersClearSelection(browserId = vm.browserId) }) {
                        Icon(
                            Icons.Filled.Close,
                            stringResource(R.string.repo_files_edit_mode_cancel_content_desc),
                        )
                    }
                }
            },
            actions = {
                TransfersButton()

                if (selectMode) {
                    IconButton(onClick = { vm.downloadSelected(navController) }) {
                        Icon(
                            Icons.Filled.Download,
                            stringResource(R.string.repo_files_edit_mode_download_selected_content_desc),
                        )
                    }

                    IconButton(onClick = { vm.mobileVault.repoFilesBrowsersDeleteSelected(browserId = vm.browserId) }) {
                        Icon(
                            Icons.Filled.Delete,
                            stringResource(R.string.repo_files_edit_mode_delete_selected_content_desc),
                        )
                    }
                }

                Box {
                    IconButton(onClick = { vm.menuExpanded.value = true }) {
                        Icon(
                            Icons.Filled.MoreVert,
                            stringResource(R.string.repo_files_nav_menu_menu_content_desc),
                        )
                    }

                    DropdownMenu(
                        expanded = vm.menuExpanded.value,
                        onDismissRequest = { vm.menuExpanded.value = false },
                    ) {
                        RepoFilesNavMenu(vm, vm.info.data)
                    }
                }
            },
        )
    }, floatingActionButton = {
        vm.info.data.value?.let { info ->
            if (info.encryptedPath != null) {
                MultiAddButton(
                    listOf(
                        MultiAddButtonItem(stringResource(R.string.repo_files_nav_menu_create_text_file_menu_item)) {
                            vm.createTextFile(context) { encryptedPath ->
                                navController.navigate(
                                    "repos/${info.repoId}/files/details?path=${
                                        queryEscape(
                                            encryptedPath,
                                        )
                                    }&editing=true",
                                )
                            }
                        },
                        MultiAddButtonItem(stringResource(R.string.repo_files_nav_menu_new_folder_menu_item)) {
                            vm.mobileVault.repoFilesBrowsersCreateDir(
                                browserId = vm.browserId,
                                cb = object : RepoFilesBrowserDirCreated {
                                    override fun onCreated(encryptedPath: String) {}
                                },
                            )
                        },
                        MultiAddButtonItem(stringResource(R.string.repo_files_nav_menu_upload_files_menu_item)) {
                            uploadFile()
                        },
                        MultiAddButtonItem(stringResource(R.string.repo_files_nav_menu_upload_folder_menu_item)) {
                            uploadFolder()
                        },
                        MultiAddButtonItem(stringResource(R.string.repo_files_nav_menu_upload_photo_menu_item)) {
                            takePicture.takePicture()
                        },
                    ),
                )
            }
        }
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        vm.info.data.value?.let { info ->
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
                    RepoFilesListRow(vm, item, selectMode, showFileInfo = {
                        vm.fileInfoSheetFile.value = item.file
                    })
                }

                // we need this spacer because the floating button can cover row
                // buttons
                item {
                    Spacer(modifier = Modifier.height(80.dp))
                }
            }
        }

        takePicture.permissionDialog()

        vm.info.data.value?.let {
            RepoFilesSortSheet(vm = vm, info = it)
        }

        RepoFileInfoSheet(
            mobileVault = vm.mobileVault,
            fileIconCache = vm.fileIconCache,
            file = vm.fileInfoSheetFile.value,
            sheetState = vm.fileInfoSheetState.value,
            onDismiss = {
                vm.fileInfoSheetFile.value = null
            },
        )
    }
}
