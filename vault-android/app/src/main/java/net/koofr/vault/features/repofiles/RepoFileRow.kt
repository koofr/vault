package net.koofr.vault.features.repofiles

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.res.stringResource
import net.koofr.vault.R
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFileType
import net.koofr.vault.features.files.FileRow

@Composable
fun RepoFileRow(
    file: RepoFile,
    fileIconBitmap: ImageBitmap,
    checkboxChecked: Boolean,
    modifiedDisplay: String?,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onMoreClick: (() -> Unit)? = null,
    moreDropdown: (@Composable () -> Unit)? = null,
    onCheckboxCheckedChange: ((Boolean) -> Unit)? = null,
) {
    FileRow(
        checkboxChecked = checkboxChecked,
        fileIcon = {
            Image(
                fileIconBitmap,
                null,
            )
        },
        name = file.name,
        contentDescription = when (file.typ) {
            RepoFileType.DIR -> stringResource(R.string.repo_files_row_folder_content_desc, file.name)
            RepoFileType.FILE -> stringResource(R.string.repo_files_row_file_content_desc, file.name)
        },
        sizeDisplay = file.sizeDisplay,
        modifiedDisplay = when (file.typ) {
            RepoFileType.DIR -> null
            RepoFileType.FILE -> modifiedDisplay
        },
        isError = file.nameError != null,
        onClick = onClick,
        onLongClick = onLongClick,
        onMoreClick = onMoreClick,
        moreDropdown = moreDropdown,
        onCheckboxCheckedChange = onCheckboxCheckedChange,
    )
}
