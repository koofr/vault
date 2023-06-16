package net.koofr.vault.features.notifications

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.features.mobilevault.subscribe
import javax.inject.Inject

@HiltViewModel
class NotificationHandlerViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel()

@Composable
fun NotificationHandler(vm: NotificationHandlerViewModel = hiltViewModel()) {
    val snackbarHostState = LocalSnackbarHostState.current
    val coroutineScope = rememberCoroutineScope()

    subscribe({ v, cb -> v.notificationsSubscribe(cb) }, { v, id ->
        val notifications = v.notificationsData(id)

        if (!notifications.isNullOrEmpty()) {
            val notification = notifications.first()

            coroutineScope.launch {
                vm.mobileVault.notificationsRemove(notification.id)

                Log.w("Vault", notification.message)

                snackbarHostState.showSnackbar(notification.message, withDismissAction = true)
            }
        }

        notifications
    })
}
