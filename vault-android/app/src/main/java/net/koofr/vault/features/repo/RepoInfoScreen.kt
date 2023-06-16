package net.koofr.vault.features.repo

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStore
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoState
import net.koofr.vault.features.mobilevault.AndroidSecureStorage
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.repounlock.RepoUnlockDialog
import net.koofr.vault.utils.WithCustomViewModelStore
import javax.inject.Inject

@HiltViewModel
class RepoInfoScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    androidSecureStorage: AndroidSecureStorage,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    val repoId: String = savedStateHandle.get<String>("repoId")!!

    private val biometricsHelper: RepoPasswordBiometricsHelper =
        RepoPasswordBiometricsHelper(repoId, androidSecureStorage)

    val biometricUnlockEnabled = mutableStateOf(biometricsHelper.isBiometricUnlockEnabled())

    val repoUnlockDialog = mutableStateOf<ViewModelStore?>(null)
    val setupBiometricUnlockVisible = mutableStateOf(false)

    fun updateBiometricUnlockEnabled() {
        biometricUnlockEnabled.value = biometricsHelper.isBiometricUnlockEnabled()
    }

    fun removeBiometricUnlock() {
        biometricsHelper.removeBiometricUnlock()

        updateBiometricUnlockEnabled()
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoInfoScreen(
    vm: RepoInfoScreenViewModel = hiltViewModel(),
) {
    val navController = LocalNavController.current

    val repo = subscribe(
        { v, cb -> v.reposRepoSubscribe(vm.repoId, cb) },
        { v, id -> v.reposRepoData(id) },
    )

    LaunchedEffect(Unit) {
        // if user clicks unlock and the biometric unlocks fails we need to
        // recheck if it is enabled
        vm.updateBiometricUnlockEnabled()
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(repo.value?.repo?.name ?: "")
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        repo.value?.repo?.let { repo ->
            Column(modifier = Modifier.padding(paddingValues)) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(15.dp),
                ) {
                    Column(
                        modifier = Modifier
                            .padding(0.dp, 0.dp, 10.dp, 0.dp)
                            .weight(1.0f),
                    ) {
                        Text(
                            text = repo.state.let {
                                when (it) {
                                    RepoState.LOCKED -> "Locked"
                                    RepoState.UNLOCKED -> "Unlocked"
                                }
                            },
                            style = MaterialTheme.typography.bodyLarge,
                            modifier = Modifier
                                .padding(0.dp, 0.dp, 0.dp, 2.dp),
                        )
                        Text(
                            text = "Unlock or lock the Safe Box",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                    Switch(
                        checked = repo.state.let {
                            when (it) {
                                RepoState.LOCKED -> false
                                RepoState.UNLOCKED -> true
                            }
                        },
                        onCheckedChange = {
                            if (it) {
                                vm.repoUnlockDialog.value = ViewModelStore()
                            } else {
                                vm.mobileVault.reposLockRepo(vm.repoId)
                            }
                        },
                        modifier = Modifier.semantics {
                            contentDescription = repo.state.let {
                                when (it) {
                                    RepoState.LOCKED -> "Locked"
                                    RepoState.UNLOCKED -> "Unlocked"
                                }
                            }
                        },
                    )
                }

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(15.dp),
                ) {
                    Column(
                        modifier = Modifier
                            .padding(0.dp, 0.dp, 10.dp, 0.dp)
                            .weight(1.0f),
                    ) {
                        Text(
                            text = "Biometric unlock",
                            style = MaterialTheme.typography.bodyLarge,
                            modifier = Modifier
                                .padding(0.dp, 0.dp, 0.dp, 2.dp),
                        )
                        Text(
                            text = "Use biometrics to unlock the Safe Box",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                    Switch(
                        checked = vm.biometricUnlockEnabled.value,
                        onCheckedChange = {
                            if (vm.biometricUnlockEnabled.value) {
                                vm.removeBiometricUnlock()
                            } else {
                                vm.setupBiometricUnlockVisible.value = true
                            }
                        },
                        modifier = Modifier.semantics {
                            contentDescription = "Biometric unlock"
                        },
                    )
                }

                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable(onClick = {
                            navController.navigate("repos/${repo.id}/remove")
                        }),
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(15.dp),
                    ) {
                        Column(
                            modifier = Modifier
                                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                                .weight(1.0f),
                        ) {
                            Text(
                                text = "Destroy Safe Box…",
                                style = MaterialTheme.typography.bodyLarge,
                                modifier = Modifier
                                    .padding(0.dp, 0.dp, 0.dp, 2.dp),
                            )
                            Text(
                                text = "Verify Safe Key and destroy the Safe Box",
                                style = MaterialTheme.typography.bodyMedium,
                            )
                        }
                    }
                }
            }
        }

        if (vm.setupBiometricUnlockVisible.value) {
            RepoSetupBiometricUnlockDialog(onDismiss = {
                vm.setupBiometricUnlockVisible.value = false

                vm.updateBiometricUnlockEnabled()
            })
        }

        vm.repoUnlockDialog.value?.let { store ->
            WithCustomViewModelStore(store) {
                RepoUnlockDialog(onDismiss = {
                    store.clear()

                    vm.repoUnlockDialog.value = null
                })
            }
        }
    }
}
