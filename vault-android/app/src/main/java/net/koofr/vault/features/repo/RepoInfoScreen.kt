package net.koofr.vault.features.repo

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStore
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoAutoLockAfter
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
    val repoAutoLockAfterDialogVisible = mutableStateOf(false)

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
        { v, cb -> v.reposRepoSubscribe(repoId = vm.repoId, cb = cb) },
        { v, id -> v.reposRepoData(id = id) },
    )

    LaunchedEffect(Unit) {
        // if user clicks unlock and the biometric unlocks fails we need to
        // recheck if it is enabled
        vm.updateBiometricUnlockEnabled()
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(
                repo.value?.repo?.name ?: "",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
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
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
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
                                vm.mobileVault.reposLockRepo(repoId = vm.repoId)
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
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
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
                            vm.repoAutoLockAfterDialogVisible.value = true
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
                                text = "Automatically lock after",
                                style = MaterialTheme.typography.bodyLarge,
                                modifier = Modifier
                                    .padding(0.dp, 0.dp, 0.dp, 2.dp),
                            )
                            Text(
                                text = repoAutoLockAfterDisplay(repo.autoLock.after),
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
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
                            text = "Lock when app hidden",
                            style = MaterialTheme.typography.bodyLarge,
                            modifier = Modifier
                                .padding(0.dp, 0.dp, 0.dp, 2.dp),
                        )
                        Text(
                            text = "When switching apps or locking the screen",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Switch(
                        checked = repo.autoLock.onAppHidden,
                        onCheckedChange = {
                            vm.mobileVault.reposSetAutoLock(
                                repo.id,
                                repo.autoLock.copy(onAppHidden = it),
                            )
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
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }

            if (vm.repoAutoLockAfterDialogVisible.value) {
                RepoAutoLockAfterDialog(
                    options = getRepoAutoLockAfterOptions(repo.autoLock.after),
                    current = repo.autoLock.after,
                    onDismiss = {
                        vm.repoAutoLockAfterDialogVisible.value = false
                    },
                    onConfirm = {
                        vm.mobileVault.reposSetAutoLock(repo.id, repo.autoLock.copy(after = it))

                        vm.repoAutoLockAfterDialogVisible.value = false
                    },
                )
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

@Composable
fun RepoAutoLockAfterDialog(
    options: List<RepoAutoLockAfter>,
    current: RepoAutoLockAfter,
    onDismiss: () -> Unit,
    onConfirm: (RepoAutoLockAfter) -> Unit,
) {
    val selected = remember { mutableStateOf(current) }

    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            TextButton(
                onClick = {
                    onConfirm(selected.value)
                },
            ) {
                Text("OK")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("CANCEL")
            }
        },
        title = { Text("Automatically lock after") },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth(),
            ) {
                LazyColumn {
                    items(options) { option ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .selectable(
                                    selected = option == selected.value,
                                    onClick = {
                                        selected.value = option
                                    },
                                ),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            RadioButton(
                                selected = option == selected.value,
                                onClick = { selected.value = option },
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(text = repoAutoLockAfterDisplay(option))
                        }
                    }
                }
            }
        },
    )
}

private fun repoAutoLockAfterDisplay(after: RepoAutoLockAfter): String {
    return when (after) {
        is RepoAutoLockAfter.NoLimit -> "No time limit"
        is RepoAutoLockAfter.Inactive1Minute -> "1 minute of inactivity"
        is RepoAutoLockAfter.Inactive5Mininutes -> "5 minutes of inactivity"
        is RepoAutoLockAfter.Inactive10Minutes -> "10 minutes of inactivity"
        is RepoAutoLockAfter.Inactive30Minutes -> "30 minutes of inactivity"
        is RepoAutoLockAfter.Inactive1Hour -> "1 hour of inactivity"
        is RepoAutoLockAfter.Inactive2Hours -> "2 hours of inactivity"
        is RepoAutoLockAfter.Inactive4Hours -> "4 hours of inactivity"
        is RepoAutoLockAfter.Custom -> "Custom (${after.seconds} seconds)"
    }
}

private fun getRepoAutoLockAfterOptions(current: RepoAutoLockAfter): List<RepoAutoLockAfter> {
    val options = mutableListOf(
        RepoAutoLockAfter.NoLimit,
        RepoAutoLockAfter.Inactive1Minute,
        RepoAutoLockAfter.Inactive5Mininutes,
        RepoAutoLockAfter.Inactive10Minutes,
        RepoAutoLockAfter.Inactive30Minutes,
        RepoAutoLockAfter.Inactive1Hour,
        RepoAutoLockAfter.Inactive2Hours,
        RepoAutoLockAfter.Inactive4Hours,
    )

    when (current) {
        is RepoAutoLockAfter.Custom -> {
            options.add(current)
        }

        else -> {}
    }

    return options
}
