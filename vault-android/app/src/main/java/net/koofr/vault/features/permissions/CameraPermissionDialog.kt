package net.koofr.vault.features.permissions

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.koofr.vault.R
import net.koofr.vault.utils.uppercaseCurrentLocale

@Composable
fun CameraPermissionDialog(
    showRationale: Boolean,
    onRequestPermission: () -> Unit,
    onCancel: () -> Unit,
) {
    AlertDialog(onDismissRequest = onCancel, title = {
        Text(stringResource(R.string.permissions_camera_title))
    }, text = {
        Text(
            if (showRationale) {
                // If the user has denied the permission but the rationale can
                // be shown, then gently explain why the app requires this
                // permission
                stringResource(R.string.permissions_camera_rationale)
            } else {
                // If it's the first time the user lands on this feature, or the
                // user doesn't want to be asked again for this permission,
                // explain that the permission is required
                stringResource(R.string.permissions_camera_required)
            },
        )
    }, confirmButton = {
        TextButton(onClick = onRequestPermission) {
            Text(stringResource(R.string.permissions_camera_request_button).uppercaseCurrentLocale())
        }
    }, dismissButton = {
        TextButton(onClick = onCancel) {
            Text(stringResource(R.string.permissions_camera_cancel_button).uppercaseCurrentLocale())
        }
    })
}
