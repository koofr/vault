package net.koofr.vault.features.repofilesdetails

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
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
import androidx.compose.runtime.mutableStateOf
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
import androidx.core.content.FileProvider
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.media3.exoplayer.ExoPlayer
import androidx.navigation.NavController
import coil.ImageLoader
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.launch
import net.koofr.vault.BuildConfig
import net.koofr.vault.FileCategory
import net.koofr.vault.FilesFilter
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesDetailsInfo
import net.koofr.vault.RepoFilesDetailsOptions
import net.koofr.vault.TransfersDownloadDone
import net.koofr.vault.composables.VideoPlayer
import net.koofr.vault.composables.ZoomableImage
import net.koofr.vault.composables.buildExoPlayer
import net.koofr.vault.features.downloads.DownloadHelper
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.storage.StorageHelper
import net.koofr.vault.features.transfers.TransferInfoView
import net.koofr.vault.features.transfers.TransfersButton
import java.io.Closeable
import java.io.File
import javax.inject.Inject

sealed class RepoFilesDetailsScreenContentData : Closeable {
    data class Text(val text: String) : RepoFilesDetailsScreenContentData()

    data class Image(
        val localFile: File,
        val ext: String,
        val imageLoader: ImageLoader,
    ) : RepoFilesDetailsScreenContentData() {
        companion object {
            val exts = setOf(
                "bmp",
                "cur",
                "gif",
                "heic",
                "ico",
                "jpeg",
                "jpg",
                "png",
                "svg",
                "webp",
            )
        }
    }

    data class Media(val exoPlayer: ExoPlayer) : RepoFilesDetailsScreenContentData() {
        override fun close() {
            exoPlayer.release()
        }

        companion object {
            val exts = setOf(
                "3gp",
                "aac",
                "amr",
                "flac",
                "imy",
                "m4a",
                "mid",
                "mkv",
                "mov",
                "mp3",
                "mp4",
                "mxmf",
                "ogg",
                "ota",
                "rtttl",
                "rtx",
                "wav",
                "webm",
                "xmf",
            )
        }
    }

    override fun close() {}

    companion object {
        fun getLoader(
            context: Context,
            file: RepoFile,
            imageLoader: ImageLoader,
        ): ((File) -> RepoFilesDetailsScreenContentData)? {
            file.ext?.let { ext ->
                if (Image.exts.contains(ext)) {
                    return {
                        Image(it, ext, imageLoader)
                    }
                } else if (Media.exts.contains(ext)) {
                    return {
                        Media(buildExoPlayer(context, Uri.fromFile(it)))
                    }
                }
            }

            if (file.category == FileCategory.TEXT || file.category == FileCategory.CODE) {
                return {
                    Text(it.readText())
                }
            }

            return null
        }
    }
}

sealed class RepoFilesDetailsScreenContent : Closeable {
    data object Loading : RepoFilesDetailsScreenContent()

    data object Downloading : RepoFilesDetailsScreenContent()

    data class Downloaded(
        val repoFile: RepoFile,
        val localFile: File,
        val data: RepoFilesDetailsScreenContentData,
    ) :
        RepoFilesDetailsScreenContent() {
        override fun close() {
            data.close()
        }
    }

    data class NotSupported(val file: RepoFile) : RepoFilesDetailsScreenContent()

    override fun close() {}
}

@HiltViewModel
class RepoFilesDetailsScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val fileIconCache: FileIconCache,
    private val storageHelper: StorageHelper,
    private val downloadHelper: DownloadHelper,
    private val imageLoader: ImageLoader,
    savedStateHandle: SavedStateHandle,
    @SuppressLint("StaticFieldLeak") @ApplicationContext private val appContext: Context,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!
    private val encryptedPath: String = savedStateHandle.get<String>("path")!!

    val detailsId = mobileVault.repoFilesDetailsCreate(
        repoId = repoId,
        encryptedPath = encryptedPath,
        isEditing = false,
        options = RepoFilesDetailsOptions(
            loadContent = FilesFilter(
                categories = listOf(FileCategory.CODE, FileCategory.TEXT),
                exts = emptyList(),
            ),
            autosaveIntervalMs = 20000u,
        ),
    )

    var currentFile: RepoFile? = null

    val content =
        mutableStateOf<RepoFilesDetailsScreenContent>(RepoFilesDetailsScreenContent.Loading)

    override fun onCleared() {
        super.onCleared()

        content.value.close()

        mobileVault.repoFilesDetailsDestroy(detailsId = detailsId)
    }

    fun setContent(newContent: RepoFilesDetailsScreenContent) {
        content.value.close()

        content.value = newContent
    }

    fun load(file: RepoFile) {
        currentFile = file

        val loader = RepoFilesDetailsScreenContentData.getLoader(appContext, file, imageLoader)

        if (loader != null) {
            setContent(RepoFilesDetailsScreenContent.Downloading)

            mobileVault.repoFilesDetailsDownloadTempFile(
                detailsId = detailsId,
                localBasePath = storageHelper.getTempDir(),
                onDone = object : TransfersDownloadDone {
                    override fun onDone(localFilePath: String, contentType: String?) {
                        viewModelScope.launch {
                            val localFile = File(localFilePath)

                            // prevent loading race conditions
                            if (currentFile == file) {
                                val data = loader(localFile)

                                setContent(
                                    RepoFilesDetailsScreenContent.Downloaded(
                                        file,
                                        localFile,
                                        data,
                                    ),
                                )
                            }
                        }
                    }
                },
            )
        } else {
            setContent(RepoFilesDetailsScreenContent.NotSupported(file))
        }
    }

    fun share(context: Context, localFile: File, contentType: String?) {
        val uri = FileProvider.getUriForFile(context, BuildConfig.FILES_AUTHORITY, localFile)

        val intent = Intent().apply {
            action = Intent.ACTION_SEND
            putExtra(Intent.EXTRA_STREAM, uri)
            contentType?.let {
                type = it
            }
        }

        context.startActivity(Intent.createChooser(intent, null))
    }

    fun download(navController: NavController, file: RepoFile) {
        downloadHelper.downloadRepoFile(navController, file)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesDetailsScreen(
    vm: RepoFilesDetailsScreenViewModel = hiltViewModel(),
) {
    val context = LocalContext.current

    val info = subscribe(
        { v, cb -> v.repoFilesDetailsInfoSubscribe(detailsId = vm.detailsId, cb = cb) },
        { v, id -> v.repoFilesDetailsInfoData(id = id) },
    )
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
                info.value?.fileName ?: "",
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
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        Column(modifier = Modifier.padding(paddingValues)) {
            info.value?.let {
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
