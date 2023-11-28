package net.koofr.vault.features.repo

import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStore
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.Status
import net.koofr.vault.composables.ErrorView
import net.koofr.vault.composables.LoadingView
import net.koofr.vault.features.repounlock.RepoUnlockScreen
import net.koofr.vault.utils.WithCustomViewModelStore
import java.io.Closeable
import javax.inject.Inject

sealed class RepoGuardState : Closeable {
    data object Loading : RepoGuardState()
    data class Locked(val viewModelStore: ViewModelStore) : RepoGuardState() {
        override fun close() {
            viewModelStore.clear()
        }
    }

    data object Unlocked : RepoGuardState()
    data class Error(val error: String) : RepoGuardState()

    fun matches(isLocked: Boolean): Boolean {
        return when (this) {
            is Loading -> false
            is Locked -> isLocked
            is Unlocked -> !isLocked
            is Error -> false
        }
    }

    override fun close() {}
}

interface WithRepoGuardViewModel {
    fun setRepoGuardViewModel(repoGuardViewModel: RepoGuardViewModel)
}

@HiltViewModel
class RepoGuardViewModel @Inject constructor(
    val mobileVault: MobileVault,
    savedStateHandle: SavedStateHandle,
) :
    ViewModel() {
    val repoId: String = savedStateHandle.get<String>("repoId")!!
    val state = mutableStateOf<RepoGuardState>(RepoGuardState.Loading)

    private fun setState(state: RepoGuardState) {
        val oldState = this.state.value

        this.state.value = state

        oldState.close()
    }

    fun update(repoStatus: Status, isLocked: Boolean) {
        repoStatus.let {
            when {
                it is Status.Initial || (it is Status.Loading && !it.loaded) -> {
                    setState(RepoGuardState.Loading)
                }

                it is Status.Loading || it is Status.Loaded -> {
                    if (!state.value.matches(isLocked)) {
                        setState(
                            if (isLocked) {
                                RepoGuardState.Locked(ViewModelStore())
                            } else {
                                RepoGuardState.Unlocked
                            },
                        )
                    }
                }

                it is Status.Err -> setState(RepoGuardState.Error(it.error))

                else -> {}
            }
        }
    }
}

@Composable
fun RepoGuard(
    vm: RepoGuardViewModel,
    setupBiometricUnlockVisible: Boolean,
    content: @Composable () -> Unit,
) {
    vm.state.value.let {
        when (it) {
            is RepoGuardState.Loading -> LoadingView()
            is RepoGuardState.Locked -> {
                WithCustomViewModelStore(it.viewModelStore) {
                    RepoUnlockScreen(
                        setupBiometricUnlockVisible = setupBiometricUnlockVisible,
                    )
                }
            }

            is RepoGuardState.Unlocked -> UnlockedRepoWrapper(repoId = vm.repoId) {
                content()
            }
            is RepoGuardState.Error -> ErrorView(errorText = it.error, onRetry = {
                vm.mobileVault.load()
            })
        }
    }
}
