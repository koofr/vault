package net.koofr.vault.features.transfers

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.Transfer
import net.koofr.vault.TransferState
import net.koofr.vault.ui.theme.VaultTheme

@Composable
fun TransferInfoView(transfer: Transfer, onRetry: () -> Unit) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        when (transfer.state) {
            is TransferState.Waiting, is TransferState.Processing -> CircularProgressIndicator(
                modifier = Modifier
                    .padding(bottom = 20.dp),
            )

            else -> transfer.percentage.let {
                if (it != null) {
                    LinearProgressIndicator(
                        it.toFloat() / 100,
                        modifier = Modifier
                            .padding(bottom = 20.dp)
                            .fillMaxWidth(),
                    )
                } else {
                    CircularProgressIndicator(
                        modifier = Modifier
                            .padding(bottom = 20.dp),
                    )
                }
            }
        }

        Text(
            text = getTransferDescription(transfer.state),
            style = MaterialTheme.typography.bodyLarge.copy(textAlign = TextAlign.Center),
            overflow = TextOverflow.Visible,
        )

        transfer.sizeProgressDisplay?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.bodyLarge.copy(textAlign = TextAlign.Center),
            )
        }

        if (transfer.canRetry) {
            TextButton(onClick = onRetry) {
                Text("RETRY")
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun TransferInfoViewPreview() {
    VaultTheme {
        Column {
            TransferInfoView(
                PreviewsData.transfersList[0],
                onRetry = {},
            )

            TransferInfoView(
                PreviewsData.transfersList[2],
                onRetry = {},
            )

            TransferInfoView(
                PreviewsData.transfersList[2].copy(
                    state = TransferState.Processing,
                    canRetry = false,
                ),
                onRetry = {},
            )
        }
    }
}
