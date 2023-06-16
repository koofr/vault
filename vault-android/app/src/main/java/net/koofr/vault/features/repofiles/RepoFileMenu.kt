package net.koofr.vault.features.repofiles

import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import net.koofr.vault.RepoFilesBrowserItem
import net.koofr.vault.RepoFilesMoveMode
import net.koofr.vault.features.navigation.LocalNavController

@Composable
fun RepoFileMenu(
    vm: RepoFilesScreenViewModel,
    item: RepoFilesBrowserItem,
    isExpanded: Boolean,
    showFileInfo: () -> Unit,
    onDismiss: () -> Unit,
) {
    val navController = LocalNavController.current

    DropdownMenu(
        expanded = isExpanded,
        onDismissRequest = onDismiss,
    ) {
        DropdownMenuItem(text = { Text(text = "Info") }, onClick = {
            onDismiss()

            showFileInfo()
        })

        item.file.path?.let { path ->
            DropdownMenuItem(text = { Text(text = "Rename") }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesRenameFile(
                    item.file.repoId,
                    path,
                )
            })

            DropdownMenuItem(text = { Text(text = "Copy") }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesMoveFile(
                    item.file.repoId,
                    path,
                    RepoFilesMoveMode.COPY,
                )
            })

            DropdownMenuItem(text = { Text(text = "Move") }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesMoveFile(
                    item.file.repoId,
                    path,
                    RepoFilesMoveMode.MOVE,
                )
            })

            DropdownMenuItem(text = { Text(text = "Delete") }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesDeleteFile(
                    item.file.repoId,
                    path,
                )
            })

            DropdownMenuItem(text = { Text(text = "Download") }, onClick = {
                onDismiss()

                vm.downloadFile(navController, item.file)
            })
        }
    }
}
