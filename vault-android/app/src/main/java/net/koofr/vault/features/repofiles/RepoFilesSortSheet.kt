package net.koofr.vault.features.repofiles

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.koofr.vault.RepoFilesBrowserInfo
import net.koofr.vault.RepoFilesSortField
import net.koofr.vault.SortDirection
import net.koofr.vault.ui.theme.KoofrGreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFilesSortSheet(
    vm: RepoFilesScreenViewModel,
    info: RepoFilesBrowserInfo,
) {
    if (vm.sortSheetVisible.value) {
        ModalBottomSheet(onDismissRequest = {
            vm.sortSheetVisible.value = false
        }, sheetState = vm.sortSheetState.value) {
            Column(Modifier.padding(bottom = 40.dp)) {
                Row(
                    Modifier
                        .padding(start = 16.dp, end = 16.dp, bottom = 10.dp)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        "SORT BY",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }

                Column(Modifier.weight(weight = 1f, fill = false)) {
                    RepoFilesSortRow(
                        vm,
                        info,
                        "File name (A to Z)",
                        RepoFilesSortField.NAME,
                        SortDirection.ASC,
                    )
                    RepoFilesSortRow(
                        vm,
                        info,
                        "File name (Z to A)",
                        RepoFilesSortField.NAME,
                        SortDirection.DESC,
                    )
                    RepoFilesSortRow(
                        vm,
                        info,
                        "Size (largest first)",
                        RepoFilesSortField.SIZE,
                        SortDirection.DESC,
                    )
                    RepoFilesSortRow(
                        vm,
                        info,
                        "Size (smallest first)",
                        RepoFilesSortField.SIZE,
                        SortDirection.ASC,
                    )
                    RepoFilesSortRow(
                        vm,
                        info,
                        "Modified (newest first)",
                        RepoFilesSortField.MODIFIED,
                        SortDirection.DESC,
                    )
                    RepoFilesSortRow(
                        vm,
                        info,
                        "Modified (oldest first)",
                        RepoFilesSortField.MODIFIED,
                        SortDirection.ASC,
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun RepoFilesSortRow(
    vm: RepoFilesScreenViewModel,
    info: RepoFilesBrowserInfo,
    text: String,
    field: RepoFilesSortField,
    direction: SortDirection,
) {
    val coroutineScope = rememberCoroutineScope()
    val isActive = info.sort.field == field && info.sort.direction == direction

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .clickable(
                onClick = {
                    vm.mobileVault.repoFilesBrowsersSortBy(vm.browserId, field, direction)

                    coroutineScope
                        .launch { vm.sortSheetState.value.hide() }
                        .invokeOnCompletion {
                            if (!vm.sortSheetState.value.isVisible) {
                                vm.sortSheetVisible.value = false
                            }
                        }
                },
            )
            .padding(start = 16.dp, end = 16.dp)
            .height(45.dp)
            .fillMaxWidth(),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
            modifier = Modifier
                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                .weight(1.0f),
            color = if (isActive) KoofrGreen else Color.Unspecified,
        )

        if (isActive) {
            Icon(Icons.Filled.Check, "Checked", tint = KoofrGreen)
        }
    }
}
