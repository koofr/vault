package net.koofr.vault.features.relativetime

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import kotlinx.coroutines.delay
import net.koofr.vault.MobileVault
import java.util.Date

@Composable
fun relativeTime(mobileVault: MobileVault, value: Long, withModifier: Boolean = true): String {
    val relativeTime = remember {
        mutableStateOf(mobileVault.relativeTime(minOf(value, Date().time), withModifier))
    }
    val nextUpdate = relativeTime.value.nextUpdate

    LaunchedEffect(value, nextUpdate) {
        if (nextUpdate != null) {
            delay(maxOf(nextUpdate - Date().time, 0))

            relativeTime.value = mobileVault.relativeTime(minOf(value, Date().time), withModifier)
        }
    }

    return relativeTime.value.display
}
