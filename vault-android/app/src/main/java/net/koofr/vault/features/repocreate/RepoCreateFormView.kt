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
import androidx.compose.material.icons.automirrored.outlined.HelpOutline
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import net.koofr.vault.R
import net.koofr.vault.RepoCreateForm
import net.koofr.vault.Status
import net.koofr.vault.composables.FormInfoSheet
import net.koofr.vault.composables.RepoPasswordField
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.remotefiles.RemoteFilesBreadcrumbs

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoCreateFormView(vm: RepoCreateViewModel, form: RepoCreateForm, modifier: Modifier) {
    val context = LocalContext.current
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
                                            this.contentDescription =
                                                context.getString(R.string.repo_create_form_location_content_desc)
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
                                Text(stringResource(R.string.repo_create_form_location_label))
                            },
                        )
                    }

                    IconButton(onClick = {
                        locationInfoSheetVisible.value = true
                    }) {
                        Icon(
                            Icons.AutoMirrored.Outlined.HelpOutline,
                            stringResource(R.string.repo_create_form_location_info_button_content_desc),
                            tint = Color.DarkGray,
                        )
                    }
                }
                Spacer(modifier = Modifier.height(20.dp))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(modifier = Modifier.weight(1f, fill = false)) {
                        RepoPasswordField(
                            value = vm.passwordState.value,
                            onValueChange = {
                                vm.passwordState.value = it

                                vm.mobileVault.repoCreateSetPassword(createId = vm.createId, password = it.text)
                            },
                            passwordVisible = vm.passwordVisible.value,
                            onPasswordVisibleChange = {
                                vm.passwordVisible.value = it
                            },
                            modifier = Modifier.fillMaxWidth(),
                            placeholder = stringResource(R.string.repo_create_form_password_placeholder),
                        )
                    }

                    IconButton(onClick = {
                        safeKeyInfoSheetVisible.value = true
                    }, modifier = Modifier.padding(top = 8.dp)) {
                        Icon(
                            Icons.AutoMirrored.Outlined.HelpOutline,
                            stringResource(R.string.repo_create_form_password_info_button_content_desc),
                            tint = Color.DarkGray,
                        )
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
                                        createId = vm.createId,
                                        salt = it.text.ifEmpty { null },
                                    )
                                },
                                label = { Text(stringResource(R.string.repo_create_form_salt_label)) },
                                placeholder = { Text(stringResource(R.string.repo_create_form_salt_placeholder)) },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(180.dp)
                                    .semantics {
                                        this.contentDescription =
                                            context.getString(R.string.repo_create_form_salt_content_desc)
                                    },
                            )
                        }

                        IconButton(onClick = {
                            saltInfoSheetVisible.value = true
                        }) {
                            Icon(
                                Icons.AutoMirrored.Outlined.HelpOutline,
                                stringResource(R.string.repo_create_form_salt_info_button_content_desc),
                                tint = Color.DarkGray,
                            )
                        }
                    }

                    Row(
                        horizontalArrangement = Arrangement.Center,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        TextButton(onClick = {
                            vm.rcloneModalVisible.value = true
                        }) {
                            Text(stringResource(R.string.repo_create_form_from_rclone_config_button))
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
                            Text(stringResource(R.string.repo_create_form_show_advanced_settings_button))
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
                        Text(stringResource(R.string.repo_create_form_create_button))
                    }
                }
            }
        }
    }

    RepoCreateRcloneDialog(vm, form.fillFromRcloneConfigError)

    FormInfoSheet(
        stringResource(R.string.repo_create_form_location_info_title),
        stringResource(R.string.repo_create_form_location_info_text),
        locationInfoSheetVisible,
    )
    FormInfoSheet(
        stringResource(R.string.repo_create_form_password_info_title),
        stringResource(R.string.repo_create_form_password_info_text),
        safeKeyInfoSheetVisible,
    )
    FormInfoSheet(
        stringResource(R.string.repo_create_form_salt_info_title),
        stringResource(R.string.repo_create_form_salt_info_text),
        saltInfoSheetVisible,
    )
}
