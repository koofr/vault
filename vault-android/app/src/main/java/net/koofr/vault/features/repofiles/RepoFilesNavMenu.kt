package net.koofr.vault.features.repofiles

import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
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
                SelectionSummary.ALL -> "Deselect all"
                else -> "Select all"
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
            Text("Copy to…")
        }, onClick = {
            vm.menuExpanded.value = false

            vm.mobileVault.repoFilesBrowsersMoveSelected(
                browserId = vm.browserId,
                RepoFilesMoveMode.COPY,
            )
        })

        DropdownMenuItem(text = {
            Text("Move to…")
        }, onClick = {
            vm.menuExpanded.value = false

            vm.mobileVault.repoFilesBrowsersMoveSelected(
                browserId = vm.browserId,
                mode = RepoFilesMoveMode.MOVE,
            )
        })
    } else {
        DropdownMenuItem(text = {
            Text("Sort by…")
        }, onClick = {
            vm.menuExpanded.value = false

            vm.sortSheetVisible.value = true
        })
    }
}
