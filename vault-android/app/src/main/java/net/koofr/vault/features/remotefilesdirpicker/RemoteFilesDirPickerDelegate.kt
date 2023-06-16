package net.koofr.vault.features.remotefilesdirpicker

interface RemoteFilesDirPickerDelegate {
    fun isActive(): Boolean
    fun navigate(location: String)
    fun cancel()
    fun canSelect(mountId: String, path: String): Boolean
    fun select(mountId: String, path: String)
}
