package net.koofr.vault.features.repos

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.R
import net.koofr.vault.Status
import net.koofr.vault.composables.RefreshableList
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.recent.RecentRow
import net.koofr.vault.features.transfers.TransfersButton
import javax.inject.Inject

@HiltViewModel
class ReposScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel() {
    val redirectedToRepoCreate = mutableStateOf(false)

    val launchedCount = mutableIntStateOf(0)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReposListScreen(
    vm: ReposScreenViewModel = hiltViewModel(),
) {
    val navController = LocalNavController.current

    val repos = subscribe(
        { v, cb -> v.reposSubscribe(cb = cb) },
        { v, id -> v.reposData(id = id) },
    )

    LaunchedEffect(Unit) {
        if (vm.launchedCount.intValue > 0) {
            vm.mobileVault.load()
        }

        vm.launchedCount.intValue++
    }

    LaunchedEffect(repos.value) {
        if (!vm.redirectedToRepoCreate.value) {
            repos.value?.let { repos ->
                repos.status.let {
                    when (it) {
                        is Status.Loaded -> {
                            if (repos.repos.isEmpty()) {
                                vm.redirectedToRepoCreate.value = true

                                navController.navigate("repoCreate")
                            }
                        }

                        else -> {}
                    }
                }
            }
        }
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text(stringResource(R.string.repos_title))
        }, actions = {
            TransfersButton()

            IconButton(onClick = { navController.navigate("settings") }) {
                Icon(Icons.Filled.Settings, stringResource(R.string.repos_settings_button_content_desc))
            }
        })
    }, floatingActionButton = {
        FloatingActionButton(onClick = {
            navController.navigate("repoCreate")
        }, containerColor = MaterialTheme.colorScheme.primary, shape = CircleShape) {
            Icon(Icons.Filled.Add, stringResource(R.string.repos_create_new_content_desc))
        }
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        repos.value?.let { repos ->
            RefreshableList(
                modifier = Modifier.padding(paddingValues),
                status = repos.status,
                isEmpty = repos.repos.isEmpty(),
                onRefresh = {
                    vm.mobileVault.load()
                },
                empty = {
                    Column(verticalArrangement = Arrangement.Center) {
                        Text(
                            stringResource(R.string.repos_empty_title),
                            style = MaterialTheme.typography.headlineMedium.copy(textAlign = TextAlign.Center),
                            modifier = Modifier
                                .padding(
                                    start = 20.dp,
                                    end = 20.dp,
                                    bottom = 20.dp,
                                )
                                .fillMaxWidth(),
                        )

                        Text(
                            stringResource(R.string.repos_empty_instruction),
                            style = MaterialTheme.typography.bodyLarge.copy(textAlign = TextAlign.Center),
                            modifier = Modifier
                                .padding(start = 30.dp, end = 30.dp)
                                .fillMaxWidth(),
                        )
                    }
                },
            ) {
                items(repos.repos, { repo -> repo.id }) { repo ->
                    Column {
                        RepoRow(repo, onClick = {
                            navController.navigate("repos/${repo.id}/files")
                        }, onMoreClick = {
                            navController.navigate("repos/${repo.id}")
                        })
                        RecentRow(repoName = repo.id) {
                            navController.navigate("repos/${repo.id}/recent")
                        }
                        Spacer(modifier = Modifier.height(15.dp))
                    }
                }

                // we need this spacer because the floating button can cover row
                // buttons
                item {
                    Spacer(modifier = Modifier.height(80.dp))
                }
            }
        }
    }
}
