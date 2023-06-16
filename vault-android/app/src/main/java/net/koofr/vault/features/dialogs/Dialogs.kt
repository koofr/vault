package net.koofr.vault.features.dialogs

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.Dialog
import net.koofr.vault.DialogType
import net.koofr.vault.MobileVault
import net.koofr.vault.features.mobilevault.subscribe
import javax.inject.Inject

@HiltViewModel
class DialogsViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel() {
    private val dialogViewModels = HashMap<UInt, DialogViewModel>()

    fun getDialogViewModel(dialog: Dialog): DialogViewModel {
        return dialogViewModels.getOrPut(dialog.id) {
            DialogViewModel(
                mobileVault,
                initialInputValue = dialog.inputValue,
                initialInputValueSelected = dialog.inputValueSelected,
            )
        }
    }

    fun cleanupDialogViewModels(dialogs: List<UInt>) {
        val dialogsSet = dialogs.toSet()
        val removeIds = mutableListOf<UInt>()

        dialogViewModels.forEach {
            if (!dialogsSet.contains(it.key)) {
                removeIds.add(it.key)
            }
        }

        removeIds.forEach(dialogViewModels::remove)
    }
}

@Composable
fun Dialogs(vm: DialogsViewModel = hiltViewModel()) {
    val dialogs = subscribe({ v, cb -> v.dialogsSubscribe(cb) }, { v, id ->
        val dialogs = v.dialogsData(id)

        dialogs?.let { vm.cleanupDialogViewModels(it) }

        dialogs
    })

    dialogs.value?.let {
        it.forEach { dialogId ->
            key(dialogId) {
                DialogsDialog(vm, dialogId)
            }
        }
    }
}

@Composable
fun DialogsDialog(dialogsVm: DialogsViewModel, dialogId: UInt) {
    val dialog = subscribe(
        { v, cb -> v.dialogsDialogSubscribe(dialogId, cb) },
        { v, id -> v.dialogsDialogData(id) },
    )

    dialog.value?.let {
        DialogView(dialogsVm.getDialogViewModel(it), it)
    }
}

class DialogViewModel constructor(
    val mobileVault: MobileVault,
    initialInputValue: String,
    initialInputValueSelected: String?,
) : ViewModel() {
    val localInputValue = mutableStateOf(
        TextFieldValue(initialInputValue)
            .let { value ->
                initialInputValueSelected?.let { selected ->
                    value.copy(selection = TextRange(start = 0, end = selected.length))
                } ?: value
            },
    )

    val inputFocusRequested = mutableStateOf(false)
}

@Composable
fun DialogView(vm: DialogViewModel, dialog: Dialog) {
    val inputFocusRequester = remember { FocusRequester() }

    AlertDialog(
        onDismissRequest = {
            vm.mobileVault.dialogsCancel(dialog.id)
        },
        title = {
            Text(dialog.title)
        },
        text = {
            dialog.message?.let {
                Text(
                    text = it,
                    modifier = Modifier
                        .padding(
                            0.dp,
                            0.dp,
                            0.dp,
                            when (dialog.typ) {
                                DialogType.PROMPT -> 15.dp
                                else -> 0.dp
                            },
                        ),
                )
            }

            when (dialog.typ) {
                DialogType.PROMPT -> {
                    TextField(
                        value = vm.localInputValue.value,
                        onValueChange = {
                            vm.localInputValue.value = it

                            vm.mobileVault.dialogsSetInputValue(dialog.id, it.text)
                        },
                        modifier = Modifier.focusRequester(inputFocusRequester),
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(
                            imeAction = ImeAction.Done,
                        ),
                        keyboardActions = KeyboardActions(onDone = {
                            if (dialog.confirmButtonEnabled) {
                                vm.mobileVault.dialogsConfirm(dialog.id)
                            }
                        }),
                    )
                }

                else -> {}
            }
        },
        confirmButton = {
            TextButton(onClick = {
                vm.mobileVault.dialogsConfirm(dialog.id)
            }, enabled = dialog.confirmButtonEnabled) {
                Text(dialog.confirmButtonText.uppercase())
            }
        },
        dismissButton = dialog.cancelButtonText?.let {
            {
                TextButton(onClick = {
                    vm.mobileVault.dialogsCancel(dialog.id)
                }) {
                    Text(it.uppercase())
                }
            }
        },
    )

    LaunchedEffect(Unit) {
        when (dialog.typ) {
            DialogType.PROMPT -> {
                if (!vm.inputFocusRequested.value) {
                    vm.inputFocusRequested.value = true

                    inputFocusRequester.requestFocus()
                }
            }

            else -> {}
        }
    }
}
