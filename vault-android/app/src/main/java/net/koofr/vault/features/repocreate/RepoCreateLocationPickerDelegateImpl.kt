package net.koofr.vault.features.repocreate

import androidx.navigation.NavController
import net.koofr.vault.features.remotefilesdirpicker.RemoteFilesDirPickerDelegate

class RepoCreateLocationPickerDelegateImpl constructor(
    private val vm: RepoCreateViewModel,
    private val navController: NavController,
) : RemoteFilesDirPickerDelegate {
    override fun isActive(): Boolean {
        return vm.locationPickerActive.value
    }

    override fun navigate(location: String) {
        vm.locationPickerNavigate(navController, location)
    }

    override fun cancel() {
        vm.locationPickerActive.value = false
    }

    override fun canSelect(mountId: String, path: String): Boolean {
        return path != "/"
    }

    override fun select(mountId: String, path: String) {
        vm.setLocation(mountId, path)

        vm.locationPickerActive.value = false
    }
}
