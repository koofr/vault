package net.koofr.vault.features.sharetarget

import androidx.compose.foundation.Image
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.LocalFileType
import net.koofr.vault.R
import net.koofr.vault.features.files.FileRow
import net.koofr.vault.features.relativetime.relativeTime
import net.koofr.vault.utils.uppercaseCurrentLocale

@Composable
fun ShareTargetFilesDialog(vm: ShareTargetViewModel) {
    AlertDialog(onDismissRequest = {
        vm.hideFilesDialog()
    }, title = {
        Text(
            pluralStringResource(
                R.plurals.share_target_files_items_count_label,
                vm.files.size,
                vm.files.size,
            ),
        )
    }, text = {
        LazyColumn() {
            items(vm.files) {
                ShareTargetFilesRow(vm = vm, file = it)
            }
        }
    }, confirmButton = {
        TextButton(onClick = {
            vm.hideFilesDialog()
        }) {
            Text(stringResource(R.string.share_target_files_dismiss_button).uppercaseCurrentLocale())
        }
    })
}

@Composable
fun ShareTargetFilesRow(vm: ShareTargetViewModel, file: ShareTargetFile) {
    val fileIconBitmap = vm.fileIconCache.getIcon(
        FileIconProps(
            size = FileIconSize.SM,
            attrs = file.localFile.fileIconAttrs,
        ),
    )

    val modifiedDisplay = file.localFile.modified?.let { relativeTime(vm.mobileVault, it) }

    FileRow(
        checkboxChecked = false,
        fileIcon = {
            Image(
                fileIconBitmap,
                null,
            )
        },
        name = file.localFile.name,
        contentDescription = when (file.localFile.typ) {
            LocalFileType.DIR -> stringResource(
                R.string.share_target_files_folder_content_desc,
                file.localFile.name,
            )
            LocalFileType.FILE -> stringResource(
                R.string.share_target_files_file_content_desc,
                file.localFile.name,
            )
        },
        sizeDisplay = file.localFile.sizeDisplay,
        modifiedDisplay = modifiedDisplay,
        isError = false,
    )
}
