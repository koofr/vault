package net.koofr.vault.features.mobilevault

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.SubscriptionCallback
import javax.inject.Inject

@HiltViewModel
class SubscribeViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel()

@Composable
fun <T> subscribe(
    subscribe: (MobileVault, SubscriptionCallback) -> UInt,
    getData: (MobileVault, UInt) -> T?,
    vm: SubscribeViewModel = hiltViewModel(),
): State<T?> {
    val coroutineScope = rememberCoroutineScope()

    val subscription = remember(Unit) {
        Subscription(
            mobileVault = vm.mobileVault,
            coroutineScope = coroutineScope,
            subscribe = subscribe,
            getData = getData,
        )
    }

    DisposableEffect(Unit) {
        onDispose {
            subscription.close()
        }
    }

    return subscription.data
}
