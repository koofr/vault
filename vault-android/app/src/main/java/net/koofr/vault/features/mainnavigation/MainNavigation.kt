package net.koofr.vault.features.mainnavigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import androidx.navigation.navDeepLink
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.remotefilesdirpicker.RemoteFilesDirPickerScreen
import net.koofr.vault.features.repo.RepoGuard
import net.koofr.vault.features.repo.RepoGuardViewModel
import net.koofr.vault.features.repo.RepoInfoScreen
import net.koofr.vault.features.repocreate.RepoCreateLocationPickerDelegateImpl
import net.koofr.vault.features.repocreate.RepoCreateScreen
import net.koofr.vault.features.repocreate.RepoCreateViewModel
import net.koofr.vault.features.repofiles.RepoFilesScreen
import net.koofr.vault.features.repofiles.RepoFilesScreenViewModel
import net.koofr.vault.features.repofilesdetails.RepoFilesDetailsScreen
import net.koofr.vault.features.repofilesdetails.RepoFilesDetailsScreenViewModel
import net.koofr.vault.features.repofilesmove.RepoFilesMoveScreen
import net.koofr.vault.features.repofilesmove.RepoFilesMoveScreenViewModel
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
                deepLinks = listOf(
                    navDeepLink {
                        uriPattern = "https://vault.koofr.net/mobile/repos/{repoId}"
                    },
                ),
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
                val vm: RepoFilesScreenViewModel = hiltViewModel()
                vm.setRepoGuardViewModel(repoGuardViewModel)

                RepoGuard(repoGuardViewModel, setupBiometricUnlockVisible = true) {
                    RepoFilesScreen(vm)
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
                val repoGuardViewModel: RepoGuardViewModel = hiltViewModel()
                val vm: RepoFilesDetailsScreenViewModel = hiltViewModel()
                vm.setRepoGuardViewModel(repoGuardViewModel)

                RepoGuard(repoGuardViewModel, setupBiometricUnlockVisible = true) {
                    RepoFilesDetailsScreen(vm)
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
                val repoGuardViewModel: RepoGuardViewModel = hiltViewModel()
                val vm: RepoFilesMoveScreenViewModel = hiltViewModel()
                vm.setRepoGuardViewModel(repoGuardViewModel)

                RepoGuard(repoGuardViewModel, setupBiometricUnlockVisible = false) {
                    RepoFilesMoveScreen(vm)
                }
            }

            composable("transfers") { TransfersScreen() }

            composable("settings") { SettingsScreen() }

            composable("info") { InfoScreen() }
        }
    }
}
