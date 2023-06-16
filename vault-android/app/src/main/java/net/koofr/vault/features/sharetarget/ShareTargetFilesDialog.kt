package net.koofr.vault.features.sharetarget

import androidx.compose.foundation.Image
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.LocalFileType
import net.koofr.vault.features.files.FileRow
import net.koofr.vault.features.relativetime.relativeTime

@Composable
fun ShareTargetFilesDialog(vm: ShareTargetViewModel) {
    AlertDialog(onDismissRequest = {
        vm.hideFilesDialog()
    }, title = {
        Text(
            vm.files.size.let {
                if (it == 1) {
                    "$it item…"
                } else {
                    "$it items…"
                }
            },
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
            Text("CLOSE")
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
            LocalFileType.DIR -> "Folder ${file.localFile.name}"
            LocalFileType.FILE -> "File ${file.localFile.name}"
        },
        sizeDisplay = file.localFile.sizeDisplay,
        modifiedDisplay = modifiedDisplay,
        isError = false,
    )
}
