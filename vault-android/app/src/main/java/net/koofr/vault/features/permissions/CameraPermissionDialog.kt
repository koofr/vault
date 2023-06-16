package net.koofr.vault.features.permissions

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable

@Composable
fun CameraPermissionDialog(
    showRationale: Boolean,
    onRequestPermission: () -> Unit,
    onCancel: () -> Unit,
) {
    AlertDialog(onDismissRequest = onCancel, title = {
        Text("Camera permission required")
    }, text = {
        Text(
            if (showRationale) {
                // If the user has denied the permission but the rationale can
                // be shown, then gently explain why the app requires this
                // permission
                "The camera is important for this app. Please grant the permission."
            } else {
                // If it's the first time the user lands on this feature, or the
                // user doesn't want to be asked again for this permission,
                // explain that the permission is required
                "Camera permission required for this feature to be available. Please grant the permission"
            },
        )
    }, confirmButton = {
        TextButton(onClick = onRequestPermission) {
            Text("REQUEST PERMISSION")
        }
    }, dismissButton = {
        TextButton(onClick = onCancel) {
            Text("CANCEL")
        }
    })
}
