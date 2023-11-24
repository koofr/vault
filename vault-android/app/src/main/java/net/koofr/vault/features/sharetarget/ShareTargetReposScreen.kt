package net.koofr.vault.features.sharetarget

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.R
import net.koofr.vault.composables.RefreshableList
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.repos.RepoRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareTargetReposScreen(
    shareTargetVm: ShareTargetViewModel,
) {
    val context = LocalContext.current
    val navController = LocalNavController.current

    val repos = subscribe(
        { v, cb -> v.reposSubscribe(cb = cb) },
        { v, id -> v.reposData(id = id) },
    )

    BackHandler(true) {
        // just cancel and the following LaunchedEffect will call finish on the activity
        shareTargetVm.cancel()
    }

    LaunchedEffect(Unit) {
        shareTargetVm.mobileVault.load()
    }

    Scaffold(topBar = {
        TopAppBar(title = {
            Text("Save to ${context.resources.getString(R.string.app_name)}")
        })
    }, bottomBar = {
        ShareTargetBottomBar(shareTargetVm, uploadEnabled = false, onUploadClick = {})
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        repos.value?.let { repos ->
            RefreshableList(
                modifier = Modifier.padding(paddingValues),
                status = repos.status,
                isEmpty = repos.repos.isEmpty(),
                onRefresh = {
                    shareTargetVm.mobileVault.load()
                },
                empty = {
                    Column(verticalArrangement = Arrangement.Center) {
                        Text(
                            "No Safe Boxes yet",
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
                            "Open ${context.resources.getString(R.string.app_name)} app and create one.",
                            style = MaterialTheme.typography.bodyLarge.copy(textAlign = TextAlign.Center),
                            modifier = Modifier
                                .padding(start = 30.dp, end = 30.dp)
                                .fillMaxWidth(),
                        )
                    }
                },
            ) {
                items(repos.repos, { repo -> repo.id }) { repo ->
                    RepoRow(repo, onClick = {
                        navController.navigate("shareTarget/repos/${repo.id}/files")
                    })
                }
            }
        }
    }
}
