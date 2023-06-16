package net.koofr.vault.features.repo

import android.content.Intent
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.RepoConfig
import net.koofr.vault.ui.theme.VaultTheme

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun RepoConfigInfo(config: RepoConfig, onSave: () -> Unit) {
    val context = LocalContext.current

    val info = remember(config) {
        buildAnnotatedString {
            val normal = { text: String ->
                append(text)
            }
            val bold = { text: String ->
                withStyle(style = SpanStyle(fontWeight = FontWeight.Bold)) {
                    append(text)
                }
            }
            val monospaced = { text: String ->
                withStyle(style = SpanStyle(fontFamily = FontFamily.Monospace)) {
                    append(text)
                }
            }

            bold("Location: ")
            normal(config.location.path)
            normal("\n\n")

            bold("Filename encryption: ")
            normal("standard")
            normal("\n\n")

            bold("Encrypt directory names: ")
            normal("true")
            normal("\n\n")

            bold("Safe Key (password): ")
            normal(config.password)
            normal("\n\n")

            bold("Salt (password2): ")
            normal(config.salt ?: "")
            normal("\n\n")

            bold("rclone config: ")
            normal("\n\n")

            monospaced(config.rcloneConfig)
        }
    }
    val infoText = info.text

    Column() {
        SelectionContainer(
            modifier = Modifier.pointerInput(Unit) {
                detectTapGestures(
                    onLongPress = {
                        // mark as saved on long press (copy)
                        onSave()
                    },
                )
            },
        ) {
            Text(info)
        }

        Spacer(modifier = Modifier.height(20.dp))

        Button(onClick = {
            val intent = Intent().apply {
                action = Intent.ACTION_SEND
                putExtra(Intent.EXTRA_TEXT, infoText)
                type = "text/plain"
            }

            context.startActivity(Intent.createChooser(intent, null))

            onSave()
        }) {
            Icon(
                Icons.Filled.Share,
                "Share",
            )
            Spacer(Modifier.size(ButtonDefaults.IconSpacing))
            Text("Share…")
        }
    }
}

@Preview(showBackground = true)
@Composable
fun RepoConfigInfoPreview() {
    VaultTheme {
        Column {
            RepoConfigInfo(config = PreviewsData.repoConfig, onSave = {})
        }
    }
}
