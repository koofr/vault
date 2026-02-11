package net.koofr.vault.features.repo

import android.content.Intent
import android.text.Html
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.fromHtml
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.R
import net.koofr.vault.RepoConfig
import net.koofr.vault.ui.theme.VaultTheme

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun RepoConfigInfo(config: RepoConfig, onSave: () -> Unit) {
    val context = LocalContext.current

    val info = remember(
        config,
        context,
    ) {
        buildAnnotatedString {
            append(AnnotatedString.fromHtml(context.getString(R.string.repo_config_info_location, Html.escapeHtml(config.location.path))))
            append("\n\n")

            append(AnnotatedString.fromHtml(context.getString(R.string.repo_config_info_filename_encryption, Html.escapeHtml("standard"))))
            append("\n\n")

            append(AnnotatedString.fromHtml(context.getString(R.string.repo_config_info_encrypt_directory_names, Html.escapeHtml("true"))))
            append("\n\n")

            append(AnnotatedString.fromHtml(context.getString(R.string.repo_config_info_salt, Html.escapeHtml(config.salt ?: ""))))
            append("\n\n")

            append(AnnotatedString.fromHtml(context.getString(R.string.repo_config_info_rclone_config)))
            append("\n\n")

            withStyle(style = SpanStyle(fontFamily = FontFamily.Monospace)) {
                append(config.rcloneConfig)
            }
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
                stringResource(R.string.repo_config_info_share_button_content_desc),
            )
            Spacer(Modifier.size(ButtonDefaults.IconSpacing))
            Text(stringResource(R.string.repo_config_info_share_button))
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
