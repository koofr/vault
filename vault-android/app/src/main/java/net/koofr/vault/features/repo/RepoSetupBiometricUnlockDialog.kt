package net.koofr.vault.features.repo

import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoUnlockMode
import net.koofr.vault.RepoUnlockOptions
import net.koofr.vault.RepoUnlockUnlocked
import net.koofr.vault.features.mobilevault.AndroidSecureStorage
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.repounlock.RepoUnlockForm
import net.koofr.vault.features.repounlock.RepoUnlockFormViewModel
import net.koofr.vault.utils.getActivity
import javax.inject.Inject

@HiltViewModel
class RepoSetupBiometricUnlockDialogViewModel @Inject constructor(
    val mobileVault: MobileVault,
    androidSecureStorage: AndroidSecureStorage,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!

    val unlockId = mobileVault.repoUnlockCreate(repoId, RepoUnlockOptions(RepoUnlockMode.VERIFY))

    private val biometricsHelper: RepoPasswordBiometricsHelper =
        RepoPasswordBiometricsHelper(repoId, androidSecureStorage)

    private var biometricPrompt: BiometricPrompt? = null

    override fun onCleared() {
        super.onCleared()

        mobileVault.repoUnlockDestroy(unlockId)
    }

    fun setup(
        password: String,
        activity: FragmentActivity,
        onDismiss: () -> Unit,
    ) {
        mobileVault.repoUnlockUnlock(
            unlockId,
            password,
            object : RepoUnlockUnlocked {
                override fun onUnlocked() {
                    viewModelScope.launch {
                        try {
                            val callback = object : BiometricPrompt.AuthenticationCallback() {
                                override fun onAuthenticationError(
                                    errorCode: Int,
                                    errString: CharSequence,
                                ) {
                                    super.onAuthenticationError(errorCode, errString)

                                    if (errorCode != BiometricPrompt.ERROR_USER_CANCELED && errorCode != BiometricPrompt.ERROR_NEGATIVE_BUTTON && errorCode != BiometricPrompt.ERROR_CANCELED) {
                                        mobileVault.notificationsShow(errString.toString())
                                    }

                                    onDismiss()
                                }

                                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                                    super.onAuthenticationSucceeded(result)

                                    result.cryptoObject?.cipher?.let {
                                        biometricsHelper.enableBiometricUnlock(it, password)

                                        onDismiss()
                                    }
                                }
                            }

                            val prompt = BiometricPrompt(activity, callback)

                            biometricsHelper.getEncryptCryptoObject()?.let { cryptoObject ->
                                biometricPrompt = prompt

                                prompt.authenticate(
                                    biometricsHelper.promptInfo,
                                    cryptoObject,
                                )
                            }
                        } catch (e: Exception) {
                            mobileVault.notificationsShow("Biometric prompt error: ${e.message}")
                        }
                    }
                }
            },
        )
    }

    fun biometricPromptCancel() {
        biometricPrompt?.let {
            it.cancelAuthentication()

            biometricPrompt = null
        }
    }
}

@Composable
fun RepoSetupBiometricUnlockDialog(
    onDismiss: () -> Unit,
    vm: RepoSetupBiometricUnlockDialogViewModel = hiltViewModel(),
    unlockFormVm: RepoUnlockFormViewModel = viewModel(),
) {
    val activity = LocalContext.current.getActivity()

    val info = subscribe(
        { v, cb -> v.repoUnlockInfoSubscribe(vm.unlockId, cb) },
        { v, id -> v.repoUnlockInfoData(id) },
    )

    val unlock = remember {
        { password: String ->
            vm.setup(password, activity, onDismiss = {
                onDismiss()
                unlockFormVm.reset()
            })
        }
    }

    DisposableEffect(Unit) {
        onDispose {
            vm.biometricPromptCancel()
        }
    }

    Dialog(
        onDismissRequest = {
            onDismiss()
            unlockFormVm.reset()
        },
    ) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surface,
        ) {
            Box(
                contentAlignment = Alignment.Center,
            ) {
                LazyColumn(
                    modifier = Modifier
                        .padding(20.dp)
                        .fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    item {
                        info.value?.let { info ->
                            RepoUnlockForm(
                                unlockFormVm,
                                info,
                                unlock,
                                message = "Enter your Safe Key to setup biometric unlock",
                            )
                        }
                    }
                }

                SnackbarHost(LocalSnackbarHostState.current)
            }
        }
    }
}
