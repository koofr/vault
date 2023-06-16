package net.koofr.vault.features.remotefilesbrowsers

import androidx.compose.runtime.Composable
import net.koofr.vault.MobileVault
import net.koofr.vault.RemoteFileType
import net.koofr.vault.RemoteFilesBrowserItem
import net.koofr.vault.RemoteFilesBrowserItemType
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.files.FileRow
import net.koofr.vault.features.relativetime.relativeTime

@Composable
fun RemoteFilesBrowserItemRow(
    mobileVault: MobileVault,
    fileIconCache: FileIconCache,
    item: RemoteFilesBrowserItem,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    onMoreClick: (() -> Unit)? = null,
    moreDropdown: (@Composable () -> Unit)? = null,
    onCheckboxCheckedChange: ((Boolean) -> Unit)? = null,
) {
    val modifiedDisplay = item.modified?.let { relativeTime(mobileVault, it) }

    FileRow(
        checkboxChecked = item.isSelected,
        fileIcon = {
            RemoteFilesBrowserItemIcon(item, fileIconCache)
        },
        name = item.name,
        contentDescription = item.typ.let {
            when (it) {
                is RemoteFilesBrowserItemType.File -> when (it.typ) {
                    RemoteFileType.DIR -> "Folder ${item.name}"
                    RemoteFileType.FILE -> "File ${item.name}"
                }

                else -> item.name
            }
        },
        sizeDisplay = item.sizeDisplay,
        modifiedDisplay = modifiedDisplay,
        isError = false,
        onClick = onClick,
        onLongClick = onLongClick,
        onMoreClick = onMoreClick,
        moreDropdown = moreDropdown,
        onCheckboxCheckedChange = onCheckboxCheckedChange,
    )
}
