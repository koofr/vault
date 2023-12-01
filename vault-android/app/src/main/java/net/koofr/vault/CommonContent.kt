package net.koofr.vault

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.features.dialogs.Dialogs
import net.koofr.vault.features.notifications.NotificationHandler
import net.koofr.vault.ui.theme.VaultTheme
import javax.inject.Inject

val LocalSnackbarHostState =
    compositionLocalOf<SnackbarHostState> { error("SnackbarHostState missing") }

@HiltViewModel
class CommonContentViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel(), DefaultLifecycleObserver {
    override fun onPause(owner: LifecycleOwner) {
        super.onPause(owner)

        mobileVault.appHidden()
    }

    override fun onResume(owner: LifecycleOwner) {
        super.onResume(owner)

        mobileVault.appVisible()
    }
}

@Composable
fun CommonContent(vm: CommonContentViewModel = hiltViewModel(), content: @Composable () -> Unit) {
    val snackbarHostState = remember { SnackbarHostState() }

    val lifecycle = LocalLifecycleOwner.current.lifecycle

    DisposableEffect(lifecycle) {
        lifecycle.addObserver(vm)

        onDispose {
            lifecycle.removeObserver(vm)
        }
    }

    CompositionLocalProvider(LocalSnackbarHostState provides snackbarHostState) {
        VaultTheme {
            content()

            NotificationHandler()

            Dialogs()
        }
    }
}
