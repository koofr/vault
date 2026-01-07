package net.koofr.vault.features.repofilesdetails

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesDetailsInfo
import net.koofr.vault.composables.VideoPlayer
import net.koofr.vault.composables.ZoomableImage
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.transfers.TransferInfoView
import net.koofr.vault.features.transfers.TransfersButton

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesDetailsScreen(
    vm: RepoFilesDetailsScreenViewModel,
) {
    val context = LocalContext.current

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
            TransfersButton()

            vm.content.value.let {
                when (it) {
                    is RepoFilesDetailsScreenContent.Downloaded -> {
                        IconButton(onClick = {
                            vm.share(context, it.localFile, it.repoFile.contentType)
                        }) {
                            Icon(Icons.Filled.Share, "Share")
                        }
                    }

                    else -> {}
                }
            }

            IconButton(onClick = { vm.rename() }) {
                Icon(Icons.Filled.Edit, "Rename")
            }

            IconButton(onClick = { vm.delete() }) {
                Icon(Icons.Filled.Delete, "Delete")
            }
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        Column(modifier = Modifier.padding(paddingValues)) {
            vm.info.data.value?.let {
                RepoFilesDetailsContentView(vm, it)
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

        Text(text = "Loading")
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
        is RepoFilesDetailsScreenContentData.Text -> RepoFilesDetailsContentDownloadedTextView(data.text)

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
fun RepoFilesDetailsContentDownloadedTextView(text: String) {
    TextField(
        value = text,
        onValueChange = {},
        readOnly = true,
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
        modifier = Modifier.semantics {
            contentDescription = "File text"
        },
    )
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
        Text(text = "Not supported", modifier = Modifier.padding(bottom = 20.dp))

        TextButton(onClick = {
            vm.download(navController, file)
        }) {
            Text("DOWNLOAD")
        }
    }
}
