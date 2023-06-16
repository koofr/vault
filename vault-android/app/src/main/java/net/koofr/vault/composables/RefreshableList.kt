package net.koofr.vault.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import net.koofr.vault.Status
import net.koofr.vault.composables.pullrefresh.PullRefreshIndicator
import net.koofr.vault.composables.pullrefresh.pullRefresh
import net.koofr.vault.composables.pullrefresh.rememberPullRefreshState

@Composable
fun RefreshableList(
    modifier: Modifier,
    status: Status,
    isEmpty: Boolean,
    onRefresh: () -> Unit,
    empty: @Composable () -> Unit,
    content: LazyListScope.() -> Unit,
) {
    val pullRefreshing = remember { mutableStateOf(false) }

    val refreshing = pullRefreshing.value && status is Status.Loading && status.loaded

    val pullRefreshState = rememberPullRefreshState(
        refreshing = refreshing,
        onRefresh = {
            pullRefreshing.value = true

            onRefresh()
        },
    )

    Box(
        modifier = modifier
            .fillMaxSize()
            .pullRefresh(pullRefreshState),
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            when {
                status is Status.Initial || (status is Status.Loading && !status.loaded) -> {}

                status is Status.Loading || status is Status.Loaded || (status is Status.Err && status.loaded) -> {
                    if (isEmpty) {
                        item {
                            Column(
                                modifier = Modifier
                                    .fillParentMaxHeight(),
                                verticalArrangement = Arrangement.Center,
                            ) {
                                empty()
                            }
                        }
                    } else {
                        content()
                    }
                }

                status is Status.Err -> {
                    item {
                        Column(
                            modifier = Modifier
                                .fillParentMaxHeight(),
                            verticalArrangement = Arrangement.Center,
                        ) {
                            ErrorView(status.error, onRetry = onRefresh)
                        }
                    }
                }
            }
        }

        PullRefreshIndicator(
            refreshing = refreshing,
            state = pullRefreshState,
            modifier = Modifier.align(Alignment.TopCenter),
        )

        if (status is Status.Loading && !status.loaded) {
            LoadingView()
        }
    }
}
