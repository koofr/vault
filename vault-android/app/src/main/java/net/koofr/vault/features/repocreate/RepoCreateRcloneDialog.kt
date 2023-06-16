package net.koofr.vault.features.repocreate

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val rclonePlaceholder = """
Format:

[name]
type=crypt
remote=rcloneremote:/path
password=obscured password
password2=obscured salt
""".trimIndent()

@Composable
fun RepoCreateRcloneDialog(
    vm: RepoCreateViewModel,
    errorText: String?,
) {
    if (vm.rcloneModalVisible.value) {
        val dismiss = {
            vm.rcloneModalVisible.value = false
        }

        AlertDialog(onDismissRequest = dismiss, title = {
            Text("From rclone config")
        }, text = {
            Column() {
                errorText?.let {
                    Text(
                        it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                    Spacer(modifier = Modifier.height(10.dp))
                }

                OutlinedTextField(
                    value = vm.rcloneConfigState.value,
                    onValueChange = {
                        vm.rcloneConfigState.value = it
                    },
                    label = { Text("rclone config") },
                    placeholder = { Text(rclonePlaceholder) },
                    isError = errorText != null,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(200.dp),
                )
            }
        }, confirmButton = {
            TextButton(onClick = {
                if (vm.mobileVault.repoCreateFillFromRcloneConfig(
                        vm.createId,
                        vm.rcloneConfigState.value.text,
                    )
                ) {
                    dismiss()
                }
            }) {
                Text("FILL")
            }
        }, dismissButton = {
            TextButton(onClick = dismiss) {
                Text("CANCEL")
            }
        })
    }
}
