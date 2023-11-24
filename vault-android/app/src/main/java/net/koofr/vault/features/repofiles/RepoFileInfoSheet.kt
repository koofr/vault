package net.koofr.vault.features.repofiles

import android.text.format.DateFormat
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import net.koofr.vault.FileCategory
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoFile
import net.koofr.vault.features.fileicon.FileIconCache
import net.koofr.vault.features.relativetime.relativeTime
import java.util.Date

@Composable
private fun Label(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.labelLarge,
        modifier = Modifier.padding(bottom = 5.dp),
    )
}

@Composable
private fun Value(
    text: String,
    color: Color = Color.Unspecified,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    modifier: Modifier = Modifier,
) {
    Text(
        text,
        color = color,
        style = style,
        modifier = modifier,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoFileInfoSheet(
    mobileVault: MobileVault,
    fileIconCache: FileIconCache,
    file: RepoFile?,
    sheetState: SheetState,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current

    file?.let {
        ModalBottomSheet(onDismissRequest = onDismiss, sheetState = sheetState) {
            LazyColumn(
                modifier = Modifier
                    .padding(20.dp, 0.dp, 20.dp, 20.dp)
                    .fillMaxWidth(),
            ) {
                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.Center,
                    ) {
                        val fileIconBitmap = fileIconCache.getIcon(
                            FileIconProps(
                                size = FileIconSize.LG,
                                attrs = file.fileIconAttrs,
                            ),
                            scale = 4,
                        )

                        Image(fileIconBitmap, null, modifier = Modifier.height(136.dp))
                    }
                    Spacer(modifier = Modifier.height(20.dp))

                    Label("Name")
                    file.nameError.let {
                        if (it != null) {
                            Value(
                                "${file.name} (ERROR)",
                                color = MaterialTheme.colorScheme.error,
                                modifier = Modifier.padding(bottom = 5.dp),
                            )
                            Value(
                                it,
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodyMedium,
                            )
                        } else {
                            Value(file.name)
                        }
                    }

                    Spacer(modifier = Modifier.height(20.dp))

                    Label("Type")
                    Value(
                        when (file.category) {
                            FileCategory.GENERIC -> "File"
                            FileCategory.FOLDER -> "Folder"
                            FileCategory.ARCHIVE -> "Archive"
                            FileCategory.AUDIO -> "Audio"
                            FileCategory.CODE -> "Code"
                            FileCategory.DOCUMENT -> "Document"
                            FileCategory.IMAGE -> "Image"
                            FileCategory.PDF -> "PDF"
                            FileCategory.PRESENTATION -> "Presentation"
                            FileCategory.SHEET -> "Spreadsheet"
                            FileCategory.TEXT -> "Text"
                            FileCategory.VIDEO -> "Video"
                        },
                    )

                    if (file.sizeDisplay.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(20.dp))

                        Label("Size")
                        Value(file.sizeDisplay)
                    }

                    file.modified?.let { modified ->
                        Spacer(modifier = Modifier.height(20.dp))

                        Label("Modified")
                        Value(
                            relativeTime(mobileVault, modified, true),
                            modifier = Modifier.padding(bottom = 5.dp),
                        )
                        Date(modified).let {
                            Value(
                                "${
                                    DateFormat.getLongDateFormat(context).format(it)
                                } ${DateFormat.getTimeFormat(context).format(it)}",
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(20.dp))

                    Label("Path")
                    Value(file.decryptedPath ?: "???")

                    Spacer(modifier = Modifier.height(20.dp))

                    Label("Encrypted path")
                    Value(file.encryptedPath)

                    Spacer(modifier = Modifier.height(30.dp))
                }
            }
        }
    }
}
