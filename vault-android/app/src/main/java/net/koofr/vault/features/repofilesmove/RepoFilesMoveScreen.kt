package net.koofr.vault.features.repofilesmove

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CreateNewFolder
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
import net.koofr.vault.RepoFilesMoveMode
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
class RepoFilesMoveScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!
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

    fun setCurrentDest() {
        mobileVault.repoFilesMoveSetDestPath(encryptedDestPath = encryptedPath)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesMoveScreen(
    vm: RepoFilesMoveScreenViewModel = hiltViewModel(),
) {
    val coroutineScope = rememberCoroutineScope()
    val navController = LocalNavController.current

    val moveInfo = subscribe(
        { v, cb -> v.repoFilesMoveInfoSubscribe(cb = cb) },
        { v, id -> v.repoFilesMoveInfoData(id = id) },
    )

    LaunchedEffect(Unit) {
        vm.setCurrentDest()
    }

    BackHandler(vm.encryptedPath == "/") {
        // if we are on root, just cancel the move and the following LaunchedEffect
        // will call navController.popBackStack
        vm.mobileVault.repoFilesMoveCancel()
    }

    LaunchedEffect(moveInfo.value != null) {
        if (moveInfo.value == null) {
            navController.popBackStack()
        }
    }

    val info = subscribe(
        { v, cb -> v.repoFilesBrowsersInfoSubscribe(browserId = vm.browserId, cb = cb) },
        { v, id -> v.repoFilesBrowsersInfoData(id = id) },
    )

    moveInfo.value?.let { moveInfo ->
        Scaffold(topBar = {
            TopAppBar(title = {
                Text(
                    info.value?.title ?: "",
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }, actions = {
                IconButton(onClick = {
                    info.value?.repoId?.let { repoId ->
                        vm.mobileVault.repoFilesBrowsersCreateDir(
                            browserId = vm.browserId,
                            cb = object : RepoFilesBrowserDirCreated {
                                override fun onCreated(encryptedPath: String) {
                                    coroutineScope.launch {
                                        navController.navigate(
                                            "repos/$repoId/files/move?path=${
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
            Surface(
                color = MaterialTheme.colorScheme.surface,
                shadowElevation = 6.dp,
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .windowInsetsPadding(NavigationBarDefaults.windowInsets),
                ) {
                    Box(modifier = Modifier.weight(1.0f)) {
                        Text(
                            if (moveInfo.srcFilesCount == 1u) {
                                "${moveInfo.srcFilesCount} item"
                            } else {
                                "${moveInfo.srcFilesCount} items"
                            },
                            modifier = Modifier.padding(15.dp, 5.dp),
                        )
                    }

                    TextButton(onClick = {
                        vm.mobileVault.repoFilesMoveCancel()
                    }) {
                        Text("CANCEL", fontSize = 16.sp)
                    }

                    TextButton(onClick = {
                        vm.mobileVault.repoFilesMoveMoveFiles()
                    }, enabled = moveInfo.canMove) {
                        Text(
                            when (moveInfo.mode) {
                                RepoFilesMoveMode.COPY -> "COPY"
                                RepoFilesMoveMode.MOVE -> "MOVE"
                            },
                            fontSize = 16.sp,
                        )
                    }
                }
            }
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
                        RepoFilesListMoveRow(vm, item)
                    }
                }
            }
        }
    }
}

@Composable
fun RepoFilesListMoveRow(
    vm: RepoFilesMoveScreenViewModel,
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
                navController.navigate("repos/${file.repoId}/files/move?path=${queryEscape(file.encryptedPath)}")
            }
        },
    )
}
