package net.koofr.vault.features.repounlock

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import net.koofr.vault.RepoUnlockInfo
import net.koofr.vault.Status
import net.koofr.vault.composables.RepoPasswordField

class RepoUnlockFormViewModel : ViewModel() {
    val passwordState = mutableStateOf(TextFieldValue())
    val passwordVisible = mutableStateOf(false)
    val passwordFocusRequested = mutableStateOf(false)

    fun reset() {
        passwordState.value = TextFieldValue()
        passwordVisible.value = false
        passwordFocusRequested.value = false
    }
}

@Composable
fun RepoUnlockForm(
    vm: RepoUnlockFormViewModel,
    info: RepoUnlockInfo,
    onUnlock: (String) -> Unit,
    message: String = "Enter your Safe Key to continue",
) {
    val passwordFocusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        if (!vm.passwordFocusRequested.value) {
            vm.passwordFocusRequested.value = true

            passwordFocusRequester.requestFocus()
        }
    }

    Text(info.repoName ?: "", style = MaterialTheme.typography.displaySmall)
    Spacer(modifier = Modifier.height(20.dp))

    Text(message, style = MaterialTheme.typography.titleMedium.copy(textAlign = TextAlign.Center))
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
        modifier = Modifier.focusRequester(passwordFocusRequester),
        passwordVisible = vm.passwordVisible.value,
        onPasswordVisibleChange = { vm.passwordVisible.value = it },
        onDone = {
            onUnlock(vm.passwordState.value.text)
        },
        errorText = errorText,
    )
    Spacer(modifier = Modifier.height(20.dp))

    Button(
        onClick = {
            onUnlock(vm.passwordState.value.text)
        },
        enabled = when (info.status) {
            is Status.Loading -> false
            else -> true
        },
    ) {
        Text("Continue")
    }
}
