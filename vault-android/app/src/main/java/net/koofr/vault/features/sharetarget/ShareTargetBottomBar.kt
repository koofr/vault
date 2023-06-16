package net.koofr.vault.features.sharetarget

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.text.ClickableText
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun ShareTargetBottomBar(
    vm: ShareTargetViewModel,
    uploadEnabled: Boolean,
    onUploadClick: () -> Unit,
) {
    Surface(
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = 6.dp,
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .windowInsetsPadding(NavigationBarDefaults.windowInsets),
        ) {
            Box(modifier = Modifier.weight(1.0f)) {
                ClickableText(
                    AnnotatedString(
                        vm.files.size.let {
                            if (it == 1) {
                                "$it item…"
                            } else {
                                "$it items…"
                            }
                        },
                        spanStyle = SpanStyle(MaterialTheme.colorScheme.onSurface),
                    ),
                    modifier = Modifier.padding(15.dp, 5.dp),
                ) {
                    vm.showFilesDialog()
                }
            }

            TextButton(onClick = {
                vm.cancel()
            }) {
                Text("CANCEL", fontSize = 16.sp)
            }

            TextButton(onClick = onUploadClick, enabled = uploadEnabled) {
                Text("UPLOAD", fontSize = 16.sp)
            }
        }
    }

    if (vm.filesDialogVisible.value) {
        ShareTargetFilesDialog(vm)
    }
}
