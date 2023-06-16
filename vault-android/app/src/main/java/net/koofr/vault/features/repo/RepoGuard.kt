package net.koofr.vault.features.repo

import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.MainScope
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoInfo
import net.koofr.vault.RepoState
import net.koofr.vault.Status
import net.koofr.vault.composables.ErrorView
import net.koofr.vault.composables.LoadingView
import net.koofr.vault.features.mobilevault.Subscription
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

    val repoState: RepoState?
        get() = when (this) {
            is Loading -> null
            is Locked -> RepoState.LOCKED
            is Unlocked -> RepoState.UNLOCKED
            is Error -> null
        }

    override fun close() {}
}

@HiltViewModel
class RepoGuardViewModel @Inject constructor(
    val mobileVault: MobileVault,
    savedStateHandle: SavedStateHandle,
) :
    ViewModel() {
    val repoId: String = savedStateHandle.get<String>("repoId")!!
    val state = mutableStateOf<RepoGuardState>(RepoGuardState.Loading)

    private val repoSubscription = Subscription(
        mobileVault = mobileVault,
        coroutineScope = MainScope(),
        subscribe = { v, cb -> v.reposRepoSubscribe(repoId, cb) },
        getData = { v, id -> v.reposRepoData(id) },
    ).also {
        addCloseable(it)
    }

    init {
        repoSubscription.setOnData {
            it?.let {
                updateState(it)
            }
        }
    }

    private fun setState(state: RepoGuardState) {
        val oldState = this.state.value

        this.state.value = state

        oldState.close()
    }

    private fun updateState(info: RepoInfo) {
        info.status.let {
            when {
                it is Status.Initial || (it is Status.Loading && !it.loaded) -> {
                    setState(RepoGuardState.Loading)
                }

                it is Status.Loading || it is Status.Loaded -> info.repo.let { repo ->
                    if (repo != null) {
                        if (this.state.value.repoState != repo.state) {
                            setState(
                                when (repo.state) {
                                    RepoState.LOCKED -> RepoGuardState.Locked(ViewModelStore())
                                    RepoState.UNLOCKED -> RepoGuardState.Unlocked
                                },
                            )
                        }
                    } else {
                        setState(RepoGuardState.Loading)
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
    setupBiometricUnlockVisible: Boolean,
    vm: RepoGuardViewModel = hiltViewModel(),
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

            is RepoGuardState.Unlocked -> content()
            is RepoGuardState.Error -> ErrorView(errorText = it.error, onRetry = {
                vm.mobileVault.load()
            })
        }
    }
}
