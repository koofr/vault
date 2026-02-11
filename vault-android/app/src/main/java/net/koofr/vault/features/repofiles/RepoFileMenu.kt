package net.koofr.vault.features.repofiles

import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.koofr.vault.R
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
        DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_get_info_menu_item)) }, onClick = {
            onDismiss()

            showFileInfo()
        })

        item.file.let { file ->
            DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_rename_menu_item)) }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesRenameFile(
                    repoId = file.repoId,
                    encryptedPath = file.encryptedPath,
                )
            })

            DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_copy_menu_item)) }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesMoveFile(
                    repoId = item.file.repoId,
                    encryptedPath = file.encryptedPath,
                    mode = RepoFilesMoveMode.COPY,
                )
            })

            DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_move_menu_item)) }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesMoveFile(
                    repoId = item.file.repoId,
                    encryptedPath = file.encryptedPath,
                    mode = RepoFilesMoveMode.MOVE,
                )
            })

            DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_delete_menu_item)) }, onClick = {
                onDismiss()

                vm.mobileVault.repoFilesDeleteFile(
                    repoId = item.file.repoId,
                    encryptedPath = file.encryptedPath,
                )
            })

            DropdownMenuItem(text = { Text(text = stringResource(R.string.repo_file_menu_download_menu_item)) }, onClick = {
                onDismiss()

                vm.downloadFile(navController, item.file)
            })
        }
    }
}
