package net.koofr.vault.features.mainnavigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.remotefilesdirpicker.RemoteFilesDirPickerScreen
import net.koofr.vault.features.repo.RepoGuard
import net.koofr.vault.features.repo.RepoInfoScreen
import net.koofr.vault.features.repocreate.RepoCreateLocationPickerDelegateImpl
import net.koofr.vault.features.repocreate.RepoCreateScreen
import net.koofr.vault.features.repocreate.RepoCreateViewModel
import net.koofr.vault.features.repofiles.RepoFilesScreen
import net.koofr.vault.features.repofilesdetails.RepoFilesDetailsScreen
import net.koofr.vault.features.repofilesmove.RepoFilesMoveScreen
import net.koofr.vault.features.reporemove.RepoRemoveScreen
import net.koofr.vault.features.repos.ReposListScreen
import net.koofr.vault.features.settings.InfoScreen
import net.koofr.vault.features.settings.SettingsScreen
import net.koofr.vault.features.transfers.TransfersScreen

@Composable
fun MainNavigation() {
    val navController = rememberNavController()

    CompositionLocalProvider(LocalNavController provides navController) {
        NavHost(navController = navController, startDestination = "repos") {
            composable("repos") { ReposListScreen() }

            composable("repoCreate") {
                RepoCreateScreen()
            }
            composable(
                "repoCreate/locationPicker?location={location}",
                arguments = listOf(
                    navArgument("location") {
                        type = NavType.StringType
                        defaultValue = ""
                    },
                ),
            ) { navBackStackEntry ->
                val repoCreateEntry = remember(navBackStackEntry) {
                    navController.getBackStackEntry("repoCreate")
                }
                val repoCreateVm: RepoCreateViewModel = viewModel(repoCreateEntry)
                val delegate = RepoCreateLocationPickerDelegateImpl(repoCreateVm, navController)

                RemoteFilesDirPickerScreen(delegate)
            }

            composable(
                "repos/{repoId}",
                arguments = listOf(
                    navArgument("repoId") {
                        type = NavType.StringType
                    },
                ),
            ) {
                RepoInfoScreen()
            }

            composable(
                "repos/{repoId}/remove",
                arguments = listOf(
                    navArgument("repoId") {
                        type = NavType.StringType
                    },
                ),
            ) {
                RepoRemoveScreen()
            }

            composable(
                "repos/{repoId}/files?path={path}",
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
                RepoGuard(setupBiometricUnlockVisible = true) {
                    RepoFilesScreen()
                }
            }

            composable(
                "repos/{repoId}/files/details?path={path}",
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
                RepoGuard(setupBiometricUnlockVisible = true) {
                    RepoFilesDetailsScreen()
                }
            }

            composable(
                "repos/{repoId}/files/move?path={path}",
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
                RepoGuard(setupBiometricUnlockVisible = false) {
                    RepoFilesMoveScreen()
                }
            }

            composable("transfers") { TransfersScreen() }

            composable("settings") { SettingsScreen() }

            composable("info") { InfoScreen() }
        }
    }
}
