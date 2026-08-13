package net.koofr.vault.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import net.koofr.vault.Status

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

    val state = rememberPullToRefreshState()

    PullToRefreshBox(
        isRefreshing = refreshing,
        onRefresh = {
            pullRefreshing.value = true

            onRefresh()
        },
        modifier = modifier.fillMaxSize(),
        state = state,
        indicator = {
            Indicator(
                state = state,
                isRefreshing = refreshing,
                modifier = Modifier.align(Alignment.TopCenter),
                containerColor = MaterialTheme.colorScheme.background,
                color = MaterialTheme.colorScheme.primary
            )
        }
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

        if (status is Status.Loading && !status.loaded) {
            LoadingView()
        }
    }
}
