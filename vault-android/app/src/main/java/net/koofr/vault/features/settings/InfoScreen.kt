package net.koofr.vault.features.settings

import android.content.ActivityNotFoundException
import android.net.Uri
import android.os.Build
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.R
import net.koofr.vault.Version
import net.koofr.vault.features.mobilevault.Config
import javax.inject.Inject

@HiltViewModel
class InfoScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val config: Config,
) : ViewModel() {
    fun reportABugUrl(version: Version): String {
        val address = "support@koofr.net"
        val subject = "I Found A Bug in Vault Android app"

        val deviceName = Build.MODEL
        val deviceManufacturer = Build.MANUFACTURER
        val osVersion = Build.VERSION.SDK_INT

        @Suppress("DEPRECATION")
        val cpuAbi = Build.CPU_ABI

        val body = "App Version: ${version.gitRelease ?: "unknown"}\n" +
            "Device manufacturer: $deviceManufacturer\n" +
            "Device name: $deviceName\n" +
            "OS version: $osVersion\n" +
            "CPU ABI: $cpuAbi\n\n"

        return Uri.Builder()
            .scheme("mailto")
            .opaquePart(address).toString() +
            Uri.Builder()
                .appendQueryParameter("subject", subject)
                .appendQueryParameter("body", body)
                .toString()
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InfoScreen(vm: InfoScreenViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    val version = remember {
        vm.mobileVault.version()
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(text = "Information")
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(paddingValues),
        ) {
            item {
                SettingsButton(
                    onClick = {
                        uriHandler.openUri(vm.config.baseUrl)
                    },
                ) {
                    Column(modifier = Modifier.padding(17.dp, 0.dp)) {
                        Text(
                            text = context.resources.getString(R.string.app_name),
                            style = MaterialTheme.typography.bodyLarge,
                        )
                        Text(
                            text = vm.config.baseUrl,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        version.gitReleaseUrl?.let {
                            uriHandler.openUri(it)
                        }
                    },
                ) {
                    Text(
                        text = "Version: ${version.gitRelease ?: "unknown"}",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        version.gitRevisionUrl?.let {
                            uriHandler.openUri(it)
                        }
                    },
                ) {
                    Text(
                        text = "Git revision: ${version.gitRevision ?: "unknown"}",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        uriHandler.openUri("${vm.config.baseUrl}/legal/tos")
                    },
                ) {
                    Text(
                        text = "Terms of Service",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        uriHandler.openUri("${vm.config.baseUrl}/legal/privacy")
                    },
                ) {
                    Text(
                        text = "Privacy Policy",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        uriHandler.openUri("https://koofr.eu/help/koofr-vault/")
                    },
                ) {
                    Text(
                        text = "Help",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        try {
                            uriHandler.openUri(vm.reportABugUrl(version))
                        } catch (e: ActivityNotFoundException) {
                            vm.mobileVault.notificationsShow(message = "Please write an email to support@koofr.net")
                        }
                    },
                ) {
                    Text(
                        text = "Report a bug",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }
        }
    }
}
