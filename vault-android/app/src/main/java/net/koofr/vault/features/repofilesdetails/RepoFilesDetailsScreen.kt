package net.koofr.vault.features.repofilesdetails

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Save
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.R
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesDetailsInfo
import net.koofr.vault.Status
import net.koofr.vault.composables.ErrorView
import net.koofr.vault.composables.VideoPlayer
import net.koofr.vault.composables.ZoomableImage
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.transfers.TransferInfoView
import net.koofr.vault.features.transfers.TransfersButton
import net.koofr.vault.utils.queryEscape
import net.koofr.vault.utils.uppercaseCurrentLocale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesDetailsScreen(
    vm: RepoFilesDetailsScreenViewModel,
) {
    val context = LocalContext.current
    val navController = LocalNavController.current

    val infoData = vm.info.data.value

    LaunchedEffect(infoData?.shouldDestroy) {
        if (infoData?.shouldDestroy == true && vm.markDestroyHandled()) {
            val repoId = infoData.repoId
            val pathChain = infoData.encryptedParentPathChain

            if (repoId != null) {
                navController.popBackStack("repos", inclusive = false)

                pathChain.forEach { path ->
                    navController.navigate("repos/$repoId/files?path=${queryEscape(path)}")
                }
            }
        }
    }

    subscribe(
        { v, cb -> v.repoFilesDetailsFileSubscribe(detailsId = vm.detailsId, cb = cb) },
        { v, id ->
            val data = v.repoFilesDetailsFileData(id = id)

            data?.let { file ->
                vm.load(file)
            }

            data
        },
    )

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(
                vm.info.data.value?.fileName ?: "",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }, actions = {
            vm.info.data.value?.let { info ->
                if (info.isEditing) {
                    IconButton(onClick = { vm.save() }, enabled = info.isDirty) {
                        Icon(Icons.Filled.Save, stringResource(R.string.repo_files_details_save_content_desc))
                    }
                    IconButton(onClick = { vm.editCancel() }) {
                        Icon(Icons.Filled.Check, stringResource(R.string.repo_files_details_done_content_desc))
                    }
                }
            }

            TransfersButton()

            vm.info.data.value?.let { info ->
                if (!info.isEditing) {
                    Box {
                        IconButton(onClick = { vm.menuExpanded.value = true }) {
                            Icon(
                                Icons.Filled.MoreVert,
                                stringResource(R.string.repo_files_details_menu_button_content_desc),
                            )
                        }

                        DropdownMenu(
                            expanded = vm.menuExpanded.value,
                            onDismissRequest = { vm.menuExpanded.value = false },
                        ) {
                            RepoFilesDetailsNavMenu(vm, context)
                        }
                    }
                }
            }
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        Column(modifier = Modifier.padding(paddingValues)) {
            vm.info.data.value.let {
                if (it != null) {
                    RepoFilesDetailsContentView(vm, it)
                } else {
                    RepoFilesDetailsContentLoadingView()
                }
            }
        }
    }
}

@Composable
fun RepoFilesDetailsContentView(
    vm: RepoFilesDetailsScreenViewModel,
    info: RepoFilesDetailsInfo,
) {
    vm.content.value.let { content ->
        when (content) {
            is RepoFilesDetailsScreenContent.Loading -> RepoFilesDetailsContentLoadingView()
            is RepoFilesDetailsScreenContent.Downloading -> info.transferId.let {
                if (it != null) {
                    RepoFilesDetailsContentDownloadingTransferView(vm, it)
                } else {
                    RepoFilesDetailsContentLoadingView()
                }
            }

            is RepoFilesDetailsScreenContent.Downloaded -> RepoFilesDetailsContentDownloadedView(
                content.data,
            )

            is RepoFilesDetailsScreenContent.TextEditor -> RepoFilesDetailsContentTextEditorView(
                vm,
            )

            is RepoFilesDetailsScreenContent.NotSupported -> RepoFilesDetailsContentNotSupportedView(
                vm,
                content.file,
            )
        }
    }
}

@Composable
fun RepoFilesDetailsContentLoadingView() {
    Column(
        modifier = Modifier
            .padding(20.dp)
            .fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        CircularProgressIndicator(
            modifier = Modifier
                .padding(bottom = 20.dp),
        )

        Text(text = stringResource(R.string.repo_files_details_loading_label))
    }
}

@Composable
fun RepoFilesDetailsContentDownloadingTransferView(
    vm: RepoFilesDetailsScreenViewModel,
    transferId: UInt,
) {
    val transfer = subscribe(
        { v, cb -> v.transfersTransferSubscribe(transferId = transferId, cb = cb) },
        { v, id -> v.transfersTransferData(id = id) },
    )

    Column(
        modifier = Modifier
            .padding(20.dp)
            .fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        transfer.value?.let {
            TransferInfoView(it, onRetry = {
                vm.mobileVault.transfersRetry(id = transferId)
            })
        }
    }
}

@Composable
fun RepoFilesDetailsContentDownloadedView(data: RepoFilesDetailsScreenContentData) {
    when (data) {
        is RepoFilesDetailsScreenContentData.Image -> {
//            AsyncImage(
//                model = data.localFile,
//                contentDescription = null,
//                imageLoader = data.imageLoader
//            )
            ZoomableImage(
                imageFile = data.localFile,
                ext = data.ext,
                imageLoader = data.imageLoader,
            )
        }

        is RepoFilesDetailsScreenContentData.Media -> {
            VideoPlayer(exoPlayer = data.exoPlayer)
        }
    }
}

@Composable
fun RepoFilesDetailsContentTextEditorView(vm: RepoFilesDetailsScreenViewModel) {
    vm.info.data.value?.let { info ->
        when (val status = info.contentStatus) {
            is Status.Err -> {
                if (!status.loaded) {
                    Column(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        ErrorView(
                            errorText = status.error,
                            onRetry = {
                                vm.mobileVault.repoFilesDetailsLoadContent(detailsId = vm.detailsId)
                            },
                        )
                    }

                    return
                }
            }

            is Status.Initial -> {
                RepoFilesDetailsContentLoadingView()
                return
            }

            is Status.Loading -> {
                if (!status.loaded) {
                    RepoFilesDetailsContentLoadingView()
                    return
                }
            }

            is Status.Loaded -> {}
        }

        RepoFilesDetailsContentTextEditorTextView(vm, info = info)
    }
}

@Composable
fun RepoFilesDetailsContentTextEditorTextView(vm: RepoFilesDetailsScreenViewModel, info: RepoFilesDetailsInfo) {
    val context = LocalContext.current

    val content = subscribe(
        { v, cb -> v.repoFilesDetailsContentBytesSubscribe(detailsId = vm.detailsId, cb = cb) },
        { v, id -> v.repoFilesDetailsContentBytesData(id = id) },
    )
    val text = content.value?.toString(Charsets.UTF_8) ?: ""
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(info.isEditing) {
        if (info.isEditing) {
            focusRequester.requestFocus()
        }
    }

    Column(modifier = Modifier.fillMaxSize()) {
        if (info.isEditing) {
            RepoFilesDetailsEditorInfo(vm, info)
        }

        TextField(
            value = text,
            onValueChange = {
                vm.setText(it)
            },
            readOnly = !info.isEditing,
            colors = TextFieldDefaults.colors(
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent,
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
            ),
            textStyle = TextStyle(
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Normal,
                fontSize = 12.sp,
            ),
            // force multiline editor to prevent text jumping when adding the
            // second line (singleLine = false does not work)
            minLines = 2,
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight()
                .weight(1f)
                .focusRequester(focusRequester)
                .semantics {
                    contentDescription = context.getString(R.string.repo_files_details_text_editor_content_desc)
                },
        )
    }
}

@Composable
fun RepoFilesDetailsContentNotSupportedView(vm: RepoFilesDetailsScreenViewModel, file: RepoFile) {
    val navController = LocalNavController.current

    Column(
        modifier = Modifier
            .padding(20.dp)
            .fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = stringResource(R.string.repo_files_details_not_supported_label),
            modifier = Modifier.padding(bottom = 20.dp),
        )

        TextButton(onClick = {
            vm.download(navController, file)
        }) {
            Text(stringResource(R.string.repo_files_details_download_button).uppercaseCurrentLocale())
        }
    }
}
