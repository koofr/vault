package net.koofr.vault.features.remotefilesdirpicker

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RemoteFileType
import net.koofr.vault.RemoteFilesBrowserDirCreated
import net.koofr.vault.RemoteFilesBrowserItemType
import net.koofr.vault.RemoteFilesBrowserOptions
import net.koofr.vault.composables.EmptyFolderView
import net.koofr.vault.composables.RefreshableList
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.remotefilesbrowsers.RemoteFilesBrowserItemRow
import javax.inject.Inject

@HiltViewModel
class RemoteFilesDirPickerScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    val location = savedStateHandle.get<String>("location")!!

    val browserId = mobileVault.remoteFilesBrowsersCreate(
        location,
        options = RemoteFilesBrowserOptions(
            selectName = null,
            onlyHostedDevices = true,
        ),
    )

    override fun onCleared() {
        super.onCleared()

        mobileVault.remoteFilesBrowsersDestroy(browserId)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RemoteFilesDirPickerScreen(
    delegate: RemoteFilesDirPickerDelegate,
    vm: RemoteFilesDirPickerScreenViewModel = hiltViewModel(),
) {
    val coroutineScope = rememberCoroutineScope()
    val navController = LocalNavController.current

    LaunchedEffect(delegate.isActive()) {
        if (!delegate.isActive()) {
            navController.popBackStack()
        }
    }

    BackHandler(vm.location == "") {
        delegate.cancel()
    }

    val info = subscribe(
        { v, cb -> v.remoteFilesBrowsersInfoSubscribe(vm.browserId, cb) },
        { v, id -> v.remoteFilesBrowsersInfoData(id) },
    )

    Scaffold(
        topBar = {
            TopAppBar(title = {
                Text(info.value?.title ?: "")
            }, actions = {
                IconButton(
                    onClick = {
                        if (info.value?.canCreateDir == true) {
                            vm.mobileVault.remoteFilesBrowsersCreateDir(
                                vm.browserId,
                                object : RemoteFilesBrowserDirCreated {
                                    override fun onCreated(location: String) {
                                        coroutineScope.launch {
                                            delegate.navigate(location)
                                        }
                                    }
                                },
                            )
                        }
                    },
                    enabled = vm.location != "",
                ) {
                    Icon(Icons.Filled.CreateNewFolder, "New folder")
                }
            })
        },
        bottomBar = {
            Surface(
                color = MaterialTheme.colorScheme.surface,
                shadowElevation = 6.dp,
            ) {
                Row(
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .windowInsetsPadding(NavigationBarDefaults.windowInsets),
                ) {
                    TextButton(onClick = {
                        delegate.cancel()
                    }) {
                        Text("CANCEL", fontSize = 16.sp)
                    }

                    TextButton(
                        onClick = {
                            info.value?.let { it ->
                                if (it.mountId != null && it.path != null) {
                                    delegate.select(it.mountId!!, it.path!!)
                                }
                            }
                        },
                        enabled = info.value?.let { it ->
                            if (it.mountId != null && it.path != null) {
                                delegate.canSelect(it.mountId!!, it.path!!)
                            } else {
                                false
                            }
                        } ?: false,
                    ) {
                        Text("SELECT", fontSize = 16.sp)
                    }
                }
            }
        },
        snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) },
    ) { paddingValues ->
        info.value?.let { info ->
            RefreshableList(
                modifier = Modifier.padding(paddingValues),
                status = info.status,
                isEmpty = info.items.isEmpty(),
                onRefresh = {
                    vm.mobileVault.remoteFilesBrowsersLoad(vm.browserId)
                },
                empty = {
                    EmptyFolderView()
                },
            ) {
                items(info.items, key = { it.id }) { item ->
                    val canNavigate = item.typ.let {
                        when (it) {
                            is RemoteFilesBrowserItemType.File -> when (it.typ) {
                                RemoteFileType.DIR -> true
                                RemoteFileType.FILE -> false
                            }

                            else -> true
                        }
                    }

                    RemoteFilesBrowserItemRow(
                        vm.mobileVault,
                        vm.fileIconCache,
                        item,
                        onClick = if (canNavigate) {
                            {
                                delegate.navigate(item.id)
                            }
                        } else {
                            null
                        },
                    )
                }

                item {
                    Spacer(modifier = Modifier.height(80.dp))
                }
            }
        }
    }
}
