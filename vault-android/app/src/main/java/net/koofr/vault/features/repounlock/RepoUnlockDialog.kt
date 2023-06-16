package net.koofr.vault.features.repounlock

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog

@Composable
fun RepoUnlockDialog(
    onDismiss: () -> Unit,
) {
    Dialog(
        onDismissRequest = {
            onDismiss()
        },
    ) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surface,
        ) {
            Box(
                contentAlignment = Alignment.Center,
            ) {
                LazyColumn(
                    modifier = Modifier
                        .padding(20.dp)
                        .fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    item {
                        RepoUnlockScreen(
                            setupBiometricUnlockVisible = false,
                            scaffold = false,
                            onUnlock = onDismiss,
                        )
                    }
                }
            }
        }
    }
}
