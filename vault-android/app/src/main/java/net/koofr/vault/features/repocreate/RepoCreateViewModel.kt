package net.koofr.vault.features.repocreate

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.text.input.TextFieldValue
import androidx.lifecycle.ViewModel
import androidx.navigation.NavController
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.RemoteFilesLocation
import net.koofr.vault.utils.queryEscape
import javax.inject.Inject

@HiltViewModel
class RepoCreateViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel() {
    var createId = mobileVault.repoCreateCreate()

    val locationPickerActive = mutableStateOf(false)

    val passwordState = mutableStateOf(TextFieldValue())
    val passwordVisible = mutableStateOf(false)

    val saltState = mutableStateOf(TextFieldValue())

    val rcloneModalVisible = mutableStateOf(false)
    val rcloneConfigState = mutableStateOf(TextFieldValue())

    val advancedVisible = mutableStateOf(false)

    val configSaved = mutableStateOf(false)

    override fun onCleared() {
        super.onCleared()

        mobileVault.repoCreateDestroy(createId = createId)
    }

    fun retryLoad() {
        mobileVault.repoCreateCreateLoad(createId = createId)
    }

    fun locationPickerNavigate(navController: NavController, location: String) {
        navController.navigate(
            "repoCreate/locationPicker?location=${
                queryEscape(
                    location,
                )
            }",
        )
    }

    fun setLocation(mountId: String, path: String) {
        mobileVault.repoCreateSetLocation(
            createId = createId,
            location = RemoteFilesLocation(mountId = mountId, path = path),
        )
    }

    fun pickLocation(navController: NavController) {
        locationPickerActive.value = true

        locationPickerNavigate(navController, "")
    }

    fun createRepo() {
        mobileVault.repoCreateCreateRepo(createId = createId)
    }

    fun saveConfig() {
        configSaved.value = true
    }
}
