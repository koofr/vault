package net.koofr.vault.features.repofiles

import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.RepoFileType
import net.koofr.vault.RepoFilesBrowserItem
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.relativetime.relativeTime
import net.koofr.vault.utils.queryEscape

@Composable
fun RepoFilesListRow(
    vm: RepoFilesScreenViewModel,
    item: RepoFilesBrowserItem,
    selectMode: Boolean,
    showFileInfo: () -> Unit,
) {
    val navController = LocalNavController.current
    val haptic = LocalHapticFeedback.current

    val fileIconBitmap = vm.fileIconCache.getIcon(
        FileIconProps(
            size = FileIconSize.SM,
            attrs = item.file.fileIconAttrs,
        ),
    )

    val modifiedDisplay = item.file.modified?.let {
        relativeTime(vm.mobileVault, it)
    }

    val menuExpanded = remember { mutableStateOf(false) }

    val selectFile = remember {
        {
            vm.mobileVault.repoFilesBrowsersSelectFile(
                browserId = vm.browserId,
                fileId = item.file.id,
                extend = true,
                range = false,
                force = false,
            )
        }
    }

    RepoFileRow(
        item.file,
        fileIconBitmap,
        modifiedDisplay = modifiedDisplay,
        checkboxChecked = item.isSelected,
        onClick = {
            if (selectMode) {
                selectFile()
            } else {
                item.file.let { file ->
                    when (item.file.typ) {
                        RepoFileType.DIR -> {
                            navController.navigate(
                                "repos/${file.repoId}/files?path=${
                                    queryEscape(
                                        file.encryptedPath,
                                    )
                                }",
                            )
                        }

                        RepoFileType.FILE -> {
                            navController.navigate(
                                "repos/${file.repoId}/files/details?path=${
                                    queryEscape(
                                        file.encryptedPath,
                                    )
                                }",
                            )
                        }
                    }
                }
            }
        },
        onLongClick = {
            selectFile()

            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
        },
        onMoreClick = if (!selectMode) {
            {
                menuExpanded.value = true
            }
        } else {
            null
        },
        moreDropdown = {
            RepoFileMenu(
                vm,
                item,
                isExpanded = menuExpanded.value,
                showFileInfo = showFileInfo,
                onDismiss = { menuExpanded.value = false },
            )
        },
        onCheckboxCheckedChange = {
            selectFile()
        },
    )
}
