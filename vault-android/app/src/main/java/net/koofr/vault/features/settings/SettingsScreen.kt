package net.koofr.vault.features.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.features.auth.AuthHelper
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.storage.StorageHelper
import net.koofr.vault.features.user.UserIcon
import java.io.IOException
import javax.inject.Inject

@HiltViewModel
class SettingsScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val authHelper: AuthHelper,
    private val storageHelper: StorageHelper,
) : ViewModel() {
    val isClearingCache = mutableStateOf(false)

    fun clearCache() {
        isClearingCache.value = true

        viewModelScope.launch(Dispatchers.IO) {
            try {
                storageHelper.clearCache()

                mobileVault.notificationsShow("Cache has been cleared")
            } catch (ex: IOException) {
                mobileVault.notificationsShow(ex.message ?: "Unknown error")
            }
        }.invokeOnCompletion {
            isClearingCache.value = false
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(vm: SettingsScreenViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val navController = LocalNavController.current

    val user = subscribe(
        { v, cb -> v.userSubscribe(cb) },
        { v, id -> v.userData(id) },
    )

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(text = "Settings")
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(paddingValues),
        ) {
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(60.dp),
                    verticalArrangement = Arrangement.Center,
                ) {
                    Row(
                        modifier = Modifier.padding(17.dp, 0.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        UserIcon()
                        Spacer(modifier = Modifier.width(15.dp))
                        Column() {
                            Text(
                                text = user.value?.fullName ?: "",
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Text(
                                text = user.value?.email ?: "",
                                style = MaterialTheme.typography.bodySmall,
                            )
                        }
                    }
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        navController.navigate("info")
                    },
                ) {
                    Column(modifier = Modifier.padding(17.dp, 0.dp)) {
                        Text(
                            text = "Information",
                            style = MaterialTheme.typography.bodyLarge,
                        )
                        Text(
                            text = "Service and application information",
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }

            item {
                SettingsButton(
                    enabled = !vm.isClearingCache.value,
                    onClick = {
                        vm.clearCache()
                    },
                ) {
                    Text(
                        text = "Clear cache",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }

            item {
                SettingsButton(
                    onClick = {
                        vm.authHelper.logout(context)
                    },
                ) {
                    Text(
                        text = "Logout",
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier
                            .padding(17.dp, 0.dp),
                    )
                }
            }
        }
    }
}

@Composable
fun SettingsButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .height(50.dp)
            .semantics { role = Role.Button }
            .alpha(if (enabled) 1f else 0.38f),
        onClick = onClick,
        enabled = enabled,
    ) {
        Column(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.Center,
        ) {
            content()
        }
    }
}
