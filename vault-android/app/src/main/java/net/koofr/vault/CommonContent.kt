package net.koofr.vault

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import net.koofr.vault.features.dialogs.Dialogs
import net.koofr.vault.features.notifications.NotificationHandler
import net.koofr.vault.ui.theme.VaultTheme

val LocalSnackbarHostState =
    compositionLocalOf<SnackbarHostState> { error("SnackbarHostState missing") }

@Composable
fun CommonContent(content: @Composable () -> Unit) {
    val snackbarHostState = remember { SnackbarHostState() }

    CompositionLocalProvider(LocalSnackbarHostState provides snackbarHostState) {
        VaultTheme {
            content()

            NotificationHandler()

            Dialogs()
        }
    }
}
