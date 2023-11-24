package net.koofr.vault.features.shareactivity

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.R
import net.koofr.vault.Status
import net.koofr.vault.features.loading.LoadingScreen
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.features.sharetarget.ShareTargetNavigation
import net.koofr.vault.features.transfers.TransfersView

@Composable
fun ShareActivityScreen(vm: ShareActivityViewModel) {
    val oauth2Status = subscribe(
        { v, cb -> v.oauth2StatusSubscribe(cb = cb) },
        { v, id -> v.oauth2StatusData(id = id) },
    )

    oauth2Status.value?.let { status ->
        when (status) {
            is Status.Initial -> ShareActivityScreenUnauthenticated(vm)
            is Status.Loading -> LoadingScreen()
            is Status.Loaded -> {
                vm.state.value.let {
                    when (it) {
                        is ShareActivityState.PreparingFiles -> ShareActivityScreenPreparingFiles(vm)
                        is ShareActivityState.NoFiles -> ShareActivityScreenNoFiles(vm)
                        is ShareActivityState.ShareTarget -> ShareTargetNavigation(vm = it.vm)
                        is ShareActivityState.Transfers -> TransfersView(vm = it.vm)
                        is ShareActivityState.Done -> ShareActivityScreenDone(vm)
                    }
                }
            }

            is Status.Err -> Text("Error ${status.error}")
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareActivityScreenBase(
    actions: @Composable RowScope.() -> Unit,
    content: @Composable () -> Unit,
) {
    val context = LocalContext.current

    Scaffold(topBar = {
        TopAppBar(title = {
            Text("Save to ${context.resources.getString(R.string.app_name)}")
        }, actions = actions)
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .fillMaxSize(),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Column(
                modifier = Modifier
                    .padding(20.dp)
                    .fillMaxSize(),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                content()
            }
        }
    }
}

@Composable
fun ShareActivityScreenUnauthenticated(vm: ShareActivityViewModel) {
    val context = LocalContext.current

    ShareActivityScreenBase(actions = {
        TextButton(onClick = {
            vm.cancel()
        }) {
            Text("CLOSE")
        }
    }) {
        Text(
            "Not signed in",
            style = MaterialTheme.typography.headlineLarge,
            modifier = Modifier.padding(bottom = 10.dp),
        )
        Text("Open ${context.resources.getString(R.string.app_name)} app and sign in.")
    }
}

@Composable
fun ShareActivityScreenPreparingFiles(vm: ShareActivityViewModel) {
    ShareActivityScreenBase(actions = {
        TextButton(onClick = {
            vm.cancel()
        }) {
            Text("CLOSE")
        }
    }) {
        CircularProgressIndicator(
            modifier = Modifier
                .padding(bottom = 20.dp),
        )

        Text("Preparing files")
    }
}

@Composable
fun ShareActivityScreenNoFiles(vm: ShareActivityViewModel) {
    ShareActivityScreenBase(actions = {
        TextButton(onClick = {
            vm.cancel()
        }) {
            Text("CLOSE")
        }
    }) {
        Text("No files to upload.")
    }
}

@Composable
fun ShareActivityScreenDone(vm: ShareActivityViewModel) {
    val progress = remember { mutableIntStateOf(0) }

    LaunchedEffect(Unit) {
        for (i in 0 until 100) {
            progress.value += 1

            delay(25)
        }

        vm.done()
    }

    ShareActivityScreenBase(actions = {
        TextButton(onClick = {
            vm.done()
        }) {
            Text("CLOSE")
        }
    }) {
        Text(
            "Upload successful",
            style = MaterialTheme.typography.headlineLarge,
            modifier = Modifier.padding(bottom = 30.dp),
        )

        LinearProgressIndicator(
            progress.value / 100f,
        )
    }
}
