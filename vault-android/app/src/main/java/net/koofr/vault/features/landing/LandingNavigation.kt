package net.koofr.vault.features.landing

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.settings.InfoScreen

@Composable
fun LandingNavigation() {
    val navController = rememberNavController()

    CompositionLocalProvider(LocalNavController provides navController) {
        NavHost(navController = navController, startDestination = "landing") {
            composable("landing") { LandingScreen() }

            composable("info") { InfoScreen() }
        }
    }
}
