package net.koofr.vault.features.repounlock

import android.security.keystore.KeyPermanentlyInvalidatedException
import android.util.Log
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoUnlockMode
import net.koofr.vault.RepoUnlockOptions
import net.koofr.vault.RepoUnlockUnlocked
import net.koofr.vault.features.mobilevault.AndroidSecureStorage
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.repo.RepoPasswordBiometricsHelper
import net.koofr.vault.features.repo.RepoSetupBiometricUnlockDialog
import net.koofr.vault.utils.getActivity
import javax.inject.Inject

@HiltViewModel
class RepoUnlockScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    androidSecureStorage: AndroidSecureStorage,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!

    val unlockId = mobileVault.repoUnlockCreate(repoId = repoId, options = RepoUnlockOptions(RepoUnlockMode.UNLOCK)).also {
        addCloseable {
            mobileVault.repoUnlockDestroy(unlockId = it)
        }
    }

    private val biometricsHelper: RepoPasswordBiometricsHelper =
        RepoPasswordBiometricsHelper(repoId, androidSecureStorage)

    private var biometricPrompt: BiometricPrompt? = null

    val canSetupBiometricUnlock = mutableStateOf(false)
    val setupBiometricUnlockVisible = mutableStateOf(false)

    fun unlock(password: String, onUnlock: (() -> Unit)?) {
        mobileVault.repoUnlockUnlock(
            unlockId = unlockId,
            password = password,
            cb = object : RepoUnlockUnlocked {
                override fun onUnlocked() {
                    viewModelScope.launch {
                        onUnlock?.invoke()
                    }
                }
            },
        )
    }

    fun biometricUnlock(activity: FragmentActivity, onUnlock: (() -> Unit)?) {
        biometricsHelper.getEncryptedPasswordIv().let {
            if (it != null) {
                val (encryptedPassword, iv) = it
                try {
                    val callback = object : BiometricPrompt.AuthenticationCallback() {
                        override fun onAuthenticationError(
                            errorCode: Int,
                            errString: CharSequence,
                        ) {
                            super.onAuthenticationError(errorCode, errString)

                            if (errorCode != BiometricPrompt.ERROR_USER_CANCELED && errorCode != BiometricPrompt.ERROR_NEGATIVE_BUTTON && errorCode != BiometricPrompt.ERROR_CANCELED) {
                                mobileVault.notificationsShow(message = errString.toString())
                            }

                            biometricPrompt = null
                        }

                        override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                            super.onAuthenticationSucceeded(result)

                            result.cryptoObject?.cipher?.let { cipher ->
                                val password =
                                    biometricsHelper.decryptPassword(cipher, encryptedPassword)

                                unlock(password, onUnlock)
                            }

                            biometricPrompt = null
                        }
                    }

                    val prompt = BiometricPrompt(activity, callback)

                    biometricsHelper.getDecryptCryptoObject(iv)?.let { cryptoObject ->
                        biometricPrompt = prompt

                        prompt.authenticate(
                            biometricsHelper.promptInfo,
                            cryptoObject,
                        )
                    }
                } catch (e: KeyPermanentlyInvalidatedException) {
                    mobileVault.notificationsShow(message = "Your biometric info has changed. Please setup biometric unlock again.")

                    biometricsHelper.removeBiometricUnlock()

                    canSetupBiometricUnlock.value = true
                } catch (e: Exception) {
                    Log.e("Vault", "Biometric prompt error", e)
                }
            } else {
                canSetupBiometricUnlock.value = true
            }
        }
    }

    fun biometricPromptCancel() {
        biometricPrompt?.let {
            it.cancelAuthentication()

            biometricPrompt = null
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoUnlockScreen(
    setupBiometricUnlockVisible: Boolean = false,
    scaffold: Boolean = true,
    onUnlock: (() -> Unit)? = null,
    vm: RepoUnlockScreenViewModel = hiltViewModel(),
    unlockFormVm: RepoUnlockFormViewModel = viewModel(),
) {
    val activity = LocalContext.current.getActivity()
    val coroutineScope = rememberCoroutineScope()

    val info = subscribe(
        { v, cb -> v.repoUnlockInfoSubscribe(unlockId = vm.unlockId, cb = cb) },
        { v, id -> v.repoUnlockInfoData(id = id) },
    )

    DisposableEffect(Unit) {
        vm.biometricUnlock(activity, onUnlock)

        onDispose {
            vm.biometricPromptCancel()
        }
    }

    val content: @Composable ColumnScope.() -> Unit = {
        info.value?.let { info ->
            Spacer(modifier = Modifier.weight(1.0f))

            RepoUnlockForm(unlockFormVm, info, { password ->
                vm.unlock(password, onUnlock)
            })

            Spacer(modifier = Modifier.weight(1.0f))

            if (vm.canSetupBiometricUnlock.value && setupBiometricUnlockVisible) {
                TextButton(onClick = {
                    vm.setupBiometricUnlockVisible.value = true
                }) {
                    Text("SETUP BIOMETRIC UNLOCK")
                }
            }

            if (vm.setupBiometricUnlockVisible.value) {
                RepoSetupBiometricUnlockDialog(onDismiss = {
                    vm.canSetupBiometricUnlock.value = false
                    vm.setupBiometricUnlockVisible.value = false

                    coroutineScope.launch {
                        // wait for the setup prompt to be hidden
                        delay(10)

                        vm.biometricUnlock(activity, onUnlock)
                    }
                })
            }
        }
    }

    if (scaffold) {
        Scaffold(topBar = {
            TopAppBar(title = {
                Text(
                    info.value?.repoName ?: "",
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            })
        }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
            Column(
                modifier = Modifier
                    .padding(paddingValues)
                    .imePadding()
                    .fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                content()
            }
        }
    } else {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            content()
        }
    }
}
