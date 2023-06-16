package net.koofr.vault.features.transfers

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.TransfersSummary
import net.koofr.vault.ui.theme.VaultTheme

@Composable
fun TransfersSummaryBottomBar(
    summary: TransfersSummary,
) {
    Surface(
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = 6.dp,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(15.dp, 15.dp, 15.dp, 15.dp)
                .windowInsetsPadding(NavigationBarDefaults.windowInsets),
        ) {
            Row(
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth(),
            ) {
                Column {
                    Text(
                        "${summary.doneCount} / ${summary.totalCount} done",
                        modifier = Modifier.padding(bottom = 10.dp),
                    )
                    Text(
                        summary.sizeProgressDisplay,
                    )
                }

                if (summary.isTransferring) {
                    Column(horizontalAlignment = Alignment.End) {
                        Text(
                            summary.speedDisplay,
                            modifier = Modifier.padding(bottom = 10.dp),
                        )
                        Text(
                            "${summary.remainingTimeDisplay} remaining",
                            textAlign = TextAlign.End,
                        )
                    }
                }
            }

            LinearProgressIndicator(
                summary.percentage.toFloat() / 100,
                modifier = Modifier
                    .padding(top = 15.dp)
                    .fillMaxWidth(),
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
fun TransfersSummaryBottomBarRowPreview() {
    VaultTheme {
        TransfersSummaryBottomBar(PreviewsData.transfersSummary)
    }
}
