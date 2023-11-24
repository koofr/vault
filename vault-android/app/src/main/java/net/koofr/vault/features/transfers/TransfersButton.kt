package net.koofr.vault.features.transfers

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Downloading
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController

@Composable
fun TransfersButton() {
    val navController = LocalNavController.current

    val transfersIsActive = subscribe(
        { v, cb -> v.transfersIsActiveSubscribe(cb = cb) },
        { v, id -> v.transfersIsActiveData(id = id) },
    )

    if (transfersIsActive.value == true) {
        IconButton(onClick = {
            navController.navigate("transfers")
        }) {
            Icon(Icons.Filled.Downloading, "Transfers")
        }
    }
}
