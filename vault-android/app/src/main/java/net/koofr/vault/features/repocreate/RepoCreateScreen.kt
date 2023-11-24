package net.koofr.vault.features.repocreate

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.RepoCreateInfo
import net.koofr.vault.Status
import net.koofr.vault.composables.ErrorView
import net.koofr.vault.composables.LoadingView
import net.koofr.vault.features.mobilevault.subscribe

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RepoCreateScreen(
    vm: RepoCreateViewModel = hiltViewModel(),
) {
    val info = subscribe(
        { v, cb -> v.repoCreateInfoSubscribe(createId = vm.createId, cb = cb) },
        { v, id -> v.repoCreateInfoData(id = id) },
    )

    Scaffold(topBar = {
        TopAppBar(title = {
            Text("Create a new Safe Box")
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        info.value?.let { info ->
            when (info) {
                is RepoCreateInfo.Form ->
                    info.form.createLoadStatus.let {
                        when (it) {
                            is Status.Initial, is Status.Loading -> {
                                LoadingView(modifier = Modifier.padding(paddingValues))
                            }

                            is Status.Err -> {
                                ErrorView(
                                    it.error,
                                    onRetry = vm::retryLoad,
                                    modifier = Modifier.padding(paddingValues),
                                )
                            }

                            else -> {
                                RepoCreateFormView(
                                    vm,
                                    info.form,
                                    modifier = Modifier.padding(paddingValues),
                                )
                            }
                        }
                    }

                is RepoCreateInfo.Created -> RepoCreateCreatedView(
                    vm,
                    info.created,
                    modifier = Modifier.padding(paddingValues),
                )
            }
        }
    }
}
