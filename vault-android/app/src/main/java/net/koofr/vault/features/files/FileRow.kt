package net.koofr.vault.features.files

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.composables.CircleCheckbox
import net.koofr.vault.ui.theme.VaultTheme

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun FileRow(
    checkboxChecked: Boolean,
    fileIcon: @Composable () -> Unit,
    name: String,
    contentDescription: String,
    sizeDisplay: String?,
    modifiedDisplay: String?,
    isError: Boolean,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    onMoreClick: (() -> Unit)? = null,
    moreDropdown: (@Composable () -> Unit)? = null,
    onCheckboxCheckedChange: ((Boolean) -> Unit)? = null,
) {
    var rowModifier = Modifier
        .height(60.dp)
        .fillMaxWidth()
        .semantics {
            this.contentDescription = contentDescription
        }

    if (onClick != null || onLongClick != null) {
        rowModifier = rowModifier.combinedClickable(
            onClick = {
                if (onClick != null) {
                    onClick()
                }
            },
            onLongClick = onLongClick,
        )
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = rowModifier,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier
                .width(60.dp)
                .height(60.dp)
                .padding(10.dp, 10.dp),
        ) {
            if (checkboxChecked) {
                CircleCheckbox(
                    checked = true,
                    onCheckedChange = {
                        if (onCheckboxCheckedChange != null) {
                            onCheckboxCheckedChange(it)
                        }
                    },
                    iconSize = 40.dp,
                )
            } else {
                fileIcon()
            }
        }

        Column(
            modifier = Modifier
                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                .weight(1.0f),
        ) {
            val hasSecondLine = !sizeDisplay.isNullOrEmpty() || !modifiedDisplay.isNullOrEmpty()

            Text(
                text = if (isError) "$name (ERROR)" else name,
                style = MaterialTheme.typography.bodyLarge.let {
                    if (isError) it.copy(color = MaterialTheme.colorScheme.error) else it
                },
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = if (hasSecondLine) {
                    Modifier.padding(
                        0.dp,
                        0.dp,
                        0.dp,
                        3.dp,
                    )
                } else {
                    Modifier
                },
            )

            if (hasSecondLine) {
                Text(
                    text = listOfNotNull(sizeDisplay, modifiedDisplay).joinToString(", "),
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }

        onMoreClick?.let { onMoreClick ->
            Box {
                IconButton(onClick = onMoreClick) {
                    Icon(Icons.Filled.MoreVert, "File menu")
                }
                moreDropdown?.let {
                    it()
                }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun FileRowPreview() {
    VaultTheme {
        Column {
            FileRow(
                checkboxChecked = false,
                fileIcon = {
                    Image(ImageBitmap(90, 110), null)
                },
                name = "example example example example example example example example example.jpg",
                contentDescription = "File example example example example example example example example example.jpg",
                sizeDisplay = "128.1 KB",
                modifiedDisplay = "2 minutes ago",
                isError = false,
                onClick = {},
            )

            FileRow(
                checkboxChecked = true,
                fileIcon = {
                    Image(ImageBitmap(90, 110), null)
                },
                name = "example folder",
                contentDescription = "Folder example folder",
                sizeDisplay = null,
                modifiedDisplay = null,
                isError = true,
                onClick = {},
            )
        }
    }
}
