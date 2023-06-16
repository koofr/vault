package net.koofr.vault.features.uploads

import android.Manifest
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.accompanist.permissions.shouldShowRationale
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.features.permissions.CameraPermissionDialog
import net.koofr.vault.features.storage.StorageHelper
import java.io.File
import javax.inject.Inject

@HiltViewModel
class TakePictureViewModel @Inject constructor(
    private val storageHelper: StorageHelper,
) : ViewModel() {
    private val currentFile = mutableStateOf<File?>(null)

    val permissionDialogVisible = mutableStateOf(false)

    fun newPicture(): Uri {
        reset()

        val (file, uri) = storageHelper.createImageFileUri()

        currentFile.value = file

        return uri
    }

    fun takeFile(): File? {
        return currentFile.value?.let { file ->
            currentFile.value = null

            file
        }
    }

    fun reset() {
        currentFile.value?.let { file ->
            file.delete()

            currentFile.value = null
        }
    }

    override fun onCleared() {
        super.onCleared()

        reset()
    }
}

data class TakePictureState(
    val takePicture: () -> Unit,
    val permissionDialog: @Composable () -> Unit,
)

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun takePicture(
    onPictureTaken: (File) -> Unit,
    vm: TakePictureViewModel = hiltViewModel(),
): TakePictureState {
    val takePictureLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.TakePicture()) { ok ->
            if (ok) {
                vm.takeFile()?.let { file ->
                    onPictureTaken(file)
                }
            } else {
                vm.reset()
            }
        }

    val takePictureGranted = remember {
        {
            takePictureLauncher.launch(vm.newPicture())
        }
    }

    val cameraPermissionState = rememberPermissionState(Manifest.permission.CAMERA) { ok ->
        if (ok) {
            takePictureGranted()
        } else {
            vm.reset()
        }
    }

    val takePicture = remember {
        {
            if (cameraPermissionState.status.isGranted) {
                takePictureGranted()
            } else {
                vm.permissionDialogVisible.value = true
            }
        }
    }

    val permissionDialog = @Composable {
        if (vm.permissionDialogVisible.value) {
            CameraPermissionDialog(
                showRationale = cameraPermissionState.status.shouldShowRationale,
                onRequestPermission = {
                    vm.permissionDialogVisible.value = false
                    cameraPermissionState.launchPermissionRequest()
                },
                onCancel = {
                    vm.permissionDialogVisible.value = false
                    vm.reset()
                },
            )
        }
    }

    return TakePictureState(takePicture, permissionDialog)
}
