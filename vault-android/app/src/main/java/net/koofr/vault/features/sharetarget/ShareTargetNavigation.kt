package net.koofr.vault.features.sharetarget

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.repo.RepoGuard
import net.koofr.vault.features.repo.RepoGuardViewModel

@Composable
fun ShareTargetNavigation(shareTargetViewModel: ShareTargetViewModel) {
    val navController = rememberNavController()

    CompositionLocalProvider(LocalNavController provides navController) {
        NavHost(navController = navController, startDestination = "shareTargetRepos") {
            composable("shareTargetRepos") {
                ShareTargetReposScreen(shareTargetViewModel)
            }
            composable(
                "shareTarget/repos/{repoId}/files?path={path}",
                arguments = listOf(
                    navArgument("repoId") {
                        type = NavType.StringType
                    },
                    navArgument("path") {
                        type = NavType.StringType
                        defaultValue = "/"
                    },
                ),
            ) {
                val repoGuardViewModel: RepoGuardViewModel = hiltViewModel()
                val vm: ShareTargetRepoFilesViewModel = hiltViewModel()
                vm.setRepoGuardViewModel(repoGuardViewModel)

                RepoGuard(repoGuardViewModel, setupBiometricUnlockVisible = false) {
                    ShareTargetRepoFilesScreen(shareTargetViewModel, vm)
                }
            }
        }
    }
}
