package net.koofr.vault.features.repocreate

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.HelpOutline
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import net.koofr.vault.RepoCreateForm
import net.koofr.vault.Status
import net.koofr.vault.composables.FormInfoSheet
import net.koofr.vault.composables.RepoPasswordField
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.remotefiles.RemoteFilesBreadcrumbs

private val repoCreateFormLocationInfoText = """
Location refers to a folder within your Koofr where all of your Safe Box files and folders are securely stored.

If this is your first Safe Box, the default location will be "My safe box." You can change it if you prefer.

If you already have a Safe Box or wish to use an existing folder (e.g. one created with rclone), you can select that folder.

Please note that you can only select a folder located within your Koofr.
""".trimIndent()

private val repoCreateFormSafeKeyInfoText = """
Safe Key is a password used to encrypt your files. Each Safe Box can have its own unique Safe Key.

Please be aware that once you set your Safe Key, it cannot be changed later. All the files within the Safe Box will be encrypted using this key.

IMPORTANT: Your Safe Key cannot be reset, and there is no way to recover your files if you forget it, as it is never sent to or stored on Koofr servers.
""".trimIndent()

private val repoCreateFormSaltInfoText = """
Salt is used in the key derivation process to create a unique encryption key and helps to protect against potential attacks. It will be stored on the Koofr servers in a secure manner.

A random Salt has been generated for you. If you prefer, you can leave the Salt field empty, and the default salt will be used (same as in rclone). However, it is recommended to use a unique salt for enhanced security. Using a unique salt helps to increase the complexity of the encryption process, making it more difficult for potential attackers to access the encrypted data.

If you wish to transfer the encrypted files to another service, it is necessary to also export the salt, otherwise you won't be able to decrypt your files.
""".trimIndent()

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoCreateFormView(vm: RepoCreateViewModel, form: RepoCreateForm, modifier: Modifier) {
    val navController = LocalNavController.current

    val locationInteractionSource = remember { MutableInteractionSource() }

    val locationInfoSheetVisible = remember { mutableStateOf(false) }
    val safeKeyInfoSheetVisible = remember { mutableStateOf(false) }
    val saltInfoSheetVisible = remember { mutableStateOf(false) }

    LaunchedEffect(form.password) {
        if (vm.passwordState.value.text != form.password) {
            vm.passwordState.value = vm.passwordState.value.copy(text = form.password)
        }
    }

    LaunchedEffect(form.salt) {
        if (vm.saltState.value.text != form.salt) {
            vm.saltState.value = vm.saltState.value.copy(text = form.salt ?: "")
        }
    }

    LazyColumn(
        modifier = modifier.fillMaxWidth(),
    ) {
        item {
            Column(modifier = Modifier.padding(17.dp)) {
                form.createRepoStatus.let {
                    when (it) {
                        is Status.Err -> {
                            Text(
                                it.error,
                                color = MaterialTheme.colorScheme.error,
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Spacer(modifier = Modifier.height(20.dp))
                        }

                        else -> {}
                    }
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(modifier = Modifier.weight(1f, fill = false)) {
                        OutlinedTextFieldDefaults.DecorationBox(
                            value = if (form.locationBreadcrumbs.isEmpty()) {
                                ""
                            } else {
                                "x"
                            },
                            innerTextField = {
                                Box(
                                    modifier = Modifier
                                        .clickable(
                                            interactionSource = locationInteractionSource,
                                            indication = null,
                                            onClick = {
                                                vm.pickLocation(navController)
                                            },
                                        )
                                        .fillMaxWidth()
                                        .semantics {
                                            this.contentDescription = "Location"
                                        },
                                ) {
                                    if (form.locationBreadcrumbs.isNotEmpty()) {
                                        RemoteFilesBreadcrumbs(breadcrumbs = form.locationBreadcrumbs)
                                    }
                                }
                            },
                            enabled = true,
                            singleLine = false,
                            visualTransformation = VisualTransformation.None,
                            interactionSource = locationInteractionSource,
                            label = {
                                Text("Location")
                            },
                        )
                    }

                    IconButton(onClick = {
                        locationInfoSheetVisible.value = true
                    }) {
                        Icon(Icons.Outlined.HelpOutline, "Location info", tint = Color.DarkGray)
                    }
                }
                Spacer(modifier = Modifier.height(20.dp))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(modifier = Modifier.weight(1f, fill = false)) {
                        RepoPasswordField(
                            value = vm.passwordState.value,
                            onValueChange = {
                                vm.passwordState.value = it

                                vm.mobileVault.repoCreateSetPassword(vm.createId, it.text)
                            },
                            passwordVisible = vm.passwordVisible.value,
                            onPasswordVisibleChange = {
                                vm.passwordVisible.value = it
                            },
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }

                    IconButton(onClick = {
                        safeKeyInfoSheetVisible.value = true
                    }, modifier = Modifier.padding(top = 8.dp)) {
                        Icon(Icons.Outlined.HelpOutline, "Safe Key info", tint = Color.DarkGray)
                    }
                }

                if (vm.advancedVisible.value) {
                    Spacer(modifier = Modifier.height(20.dp))

                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.weight(1f, fill = false)) {
                            OutlinedTextField(
                                value = vm.saltState.value,
                                onValueChange = {
                                    vm.saltState.value = it

                                    vm.mobileVault.repoCreateSetSalt(
                                        vm.createId,
                                        it.text.ifEmpty { null },
                                    )
                                },
                                label = { Text("Salt") },
                                placeholder = { Text("Salt") },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(180.dp)
                                    .semantics {
                                        this.contentDescription = "Salt"
                                    },
                            )
                        }

                        IconButton(onClick = {
                            saltInfoSheetVisible.value = true
                        }) {
                            Icon(Icons.Outlined.HelpOutline, "Safe Key info", tint = Color.DarkGray)
                        }
                    }

                    Row(
                        horizontalArrangement = Arrangement.Center,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        TextButton(onClick = {
                            vm.rcloneModalVisible.value = true
                        }) {
                            Text("From rclone config")
                        }
                    }
                    Spacer(modifier = Modifier.height(10.dp))
                } else {
                    Row(
                        horizontalArrangement = Arrangement.Center,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        TextButton(onClick = {
                            vm.advancedVisible.value = true
                        }) {
                            Text("Show advanced settings")
                        }
                    }
                    Spacer(modifier = Modifier.height(10.dp))
                }

                Row(
                    horizontalArrangement = Arrangement.Center,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Button(
                        onClick = {
                            vm.createRepo()
                        },
                        enabled = form.canCreate && form.createRepoStatus.let {
                            when (it) {
                                is Status.Loading -> false
                                else -> true
                            }
                        },
                    ) {
                        Text("Create")
                    }
                }
            }
        }
    }

    RepoCreateRcloneDialog(vm, form.fillFromRcloneConfigError)

    FormInfoSheet("Location", repoCreateFormLocationInfoText, locationInfoSheetVisible)
    FormInfoSheet("Safe Key", repoCreateFormSafeKeyInfoText, safeKeyInfoSheetVisible)
    FormInfoSheet("Salt", repoCreateFormSaltInfoText, saltInfoSheetVisible)
}
