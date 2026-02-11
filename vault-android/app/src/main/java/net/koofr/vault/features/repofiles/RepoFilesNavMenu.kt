package net.koofr.vault.features.repofiles

import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.ui.res.stringResource
import net.koofr.vault.R
import net.koofr.vault.RepoFilesBrowserInfo
import net.koofr.vault.RepoFilesMoveMode
import net.koofr.vault.SelectionSummary

@Composable
fun RepoFilesNavMenu(vm: RepoFilesScreenViewModel, info: State<RepoFilesBrowserInfo?>) {
    val selectedCount = info.value?.selectedCount ?: 0u
    val selectMode = selectedCount > 0u

    DropdownMenuItem(text = {
        Text(
            text = when (info.value?.selectionSummary) {
                SelectionSummary.ALL -> stringResource(R.string.repo_files_edit_mode_deselect_all_button)
                else -> stringResource(R.string.repo_files_edit_mode_select_all_button)
            },
        )
    }, onClick = {
        vm.menuExpanded.value = false

        when (info.value?.selectionSummary) {
            SelectionSummary.ALL -> vm.mobileVault.repoFilesBrowsersClearSelection(
                browserId = vm.browserId,
            )

            else -> vm.mobileVault.repoFilesBrowsersSelectAll(browserId = vm.browserId)
        }
    })

    if (selectMode) {
        DropdownMenuItem(text = {
            Text(stringResource(R.string.repo_files_nav_menu_copy_to_menu_item))
        }, onClick = {
            vm.menuExpanded.value = false

            vm.mobileVault.repoFilesBrowsersMoveSelected(
                browserId = vm.browserId,
                RepoFilesMoveMode.COPY,
            )
        })

        DropdownMenuItem(text = {
            Text(stringResource(R.string.repo_files_nav_menu_move_to_menu_item))
        }, onClick = {
            vm.menuExpanded.value = false

            vm.mobileVault.repoFilesBrowsersMoveSelected(
                browserId = vm.browserId,
                mode = RepoFilesMoveMode.MOVE,
            )
        })
    } else {
        DropdownMenuItem(text = {
            Text(stringResource(R.string.repo_files_nav_menu_sort_by_menu_item))
        }, onClick = {
            vm.menuExpanded.value = false

            vm.sortSheetVisible.value = true
        })
    }
}
