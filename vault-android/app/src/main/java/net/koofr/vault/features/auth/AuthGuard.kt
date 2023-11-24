package net.koofr.vault.features.auth

import androidx.compose.runtime.Composable
import net.koofr.vault.Status
import net.koofr.vault.features.landing.LandingNavigation
import net.koofr.vault.features.loading.LoadingScreen
import net.koofr.vault.features.mainnavigation.MainNavigation
import net.koofr.vault.features.mobilevault.subscribe

@Composable
fun AuthGuard() {
    val oauth2Status = subscribe(
        { v, cb -> v.oauth2StatusSubscribe(cb = cb) },
        { v, id -> v.oauth2StatusData(id = id) },
    )

    when (oauth2Status.value) {
        is Status.Loading -> LoadingScreen()
        is Status.Loaded -> MainNavigation()
        else -> LandingNavigation()
    }
}
