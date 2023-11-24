package net.koofr.vault.features.transfers

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController

@Composable
fun TransfersScreen() {
    val navController = LocalNavController.current

    val isActive = subscribe(
        { v, cb -> v.transfersIsActiveSubscribe(cb = cb) },
        { v, id -> v.transfersIsActiveData(id = id) },
    )

    LaunchedEffect(isActive.value) {
        if (isActive.value == false) {
            navController.popBackStack()
        }
    }

    TransfersView()
}
