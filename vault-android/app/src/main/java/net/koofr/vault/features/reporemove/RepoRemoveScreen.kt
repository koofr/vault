package net.koofr.vault.features.reporemove

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoRemoved
import net.koofr.vault.Status
import net.koofr.vault.composables.RepoPasswordField
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import javax.inject.Inject

@HiltViewModel
class RepoRemoveScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val repoId: String = savedStateHandle.get<String>("repoId")!!

    val removeId = mobileVault.repoRemoveCreate(repoId = repoId)

    val passwordState = mutableStateOf(TextFieldValue())
    val passwordVisible = mutableStateOf(false)

    override fun onCleared() {
        super.onCleared()

        mobileVault.repoRemoveDestroy(removeId = removeId)
    }

    fun remove(cb: () -> Unit) {
        mobileVault.repoRemoveRemove(
            removeId = removeId,
            password = passwordState.value.text,
            cb = object : RepoRemoved {
                override fun onRemoved() {
                    viewModelScope.launch {
                        cb()
                    }
                }
            },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoRemoveScreen(
    vm: RepoRemoveScreenViewModel = hiltViewModel(),
) {
    val navController = LocalNavController.current

    val passwordFocusRequester = remember { FocusRequester() }

    val info = subscribe(
        { v, cb -> v.repoRemoveInfoSubscribe(removeId = vm.removeId, cb = cb) },
        { v, id -> v.repoRemoveInfoData(id = id) },
    )

    LaunchedEffect(Unit) {
        passwordFocusRequester.requestFocus()
    }

    val remove = {
        vm.remove {
            navController.popBackStack("repos", false)
        }
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text("Destroy Safe Box")
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .padding(paddingValues)
                .fillMaxWidth(),
        ) {
            item {
                info.value?.let { info ->
                    if (info.repoName != null) {
                        Column(modifier = Modifier.padding(17.dp, 0.dp)) {
                            Text(
                                buildAnnotatedString {
                                    append("Do you really want to destroy Safe Box ")
                                    withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
                                        append(info.repoName)
                                    }
                                    append("?")
                                },
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Spacer(modifier = Modifier.height(20.dp))

                            Text(
                                "Destroying the Safe Box will keep all the files on Koofr but remove the configuration so you won't be able to decrypt the files if you didn't save the configuration.",
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Spacer(modifier = Modifier.height(20.dp))

                            Text(
                                "This action cannot be undone.",
                                fontWeight = FontWeight.Bold,
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Spacer(modifier = Modifier.height(20.dp))

                            Text(
                                "Enter your Safe Key to confirm the removal:",
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Spacer(modifier = Modifier.height(20.dp))

                            val errorText = info.status.let {
                                when (it) {
                                    is Status.Err -> it.error
                                    else -> null
                                }
                            }

                            RepoPasswordField(
                                value = vm.passwordState.value,
                                onValueChange = { vm.passwordState.value = it },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .focusRequester(passwordFocusRequester),
                                passwordVisible = vm.passwordVisible.value,
                                onPasswordVisibleChange = { vm.passwordVisible.value = it },
                                onDone = remove,
                                errorText = errorText,
                            )
                            Spacer(modifier = Modifier.height(20.dp))

                            Row(
                                horizontalArrangement = Arrangement.Center,
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                Button(
                                    onClick = remove,
                                    colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error),
                                    enabled = when (info.status) {
                                        is Status.Loading -> false
                                        else -> true
                                    },
                                ) {
                                    Text("Destroy")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
