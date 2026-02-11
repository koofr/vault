package net.koofr.vault.features.transfers

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.R
import net.koofr.vault.Transfer
import net.koofr.vault.TransferState
import net.koofr.vault.ui.theme.VaultTheme
import net.koofr.vault.utils.uppercaseCurrentLocale

@Composable
fun TransferRow(
    transfer: Transfer,
    fileIconBitmap: ImageBitmap,
    onRetry: () -> Unit,
    onAbort: () -> Unit,
    onOpen: () -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .defaultMinSize(minHeight = 60.dp)
            .fillMaxWidth(),
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier
                .width(60.dp)
                .height(60.dp)
                .padding(10.dp, 10.dp),
        ) {
            Image(
                fileIconBitmap,
                null,
            )
        }

        Column(
            modifier = Modifier
                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                .weight(1.0f),
        ) {
            Text(
                text = transfer.name,
                style = MaterialTheme.typography.bodyLarge,
                overflow = TextOverflow.Visible,
                modifier = Modifier
                    .padding(0.dp, 0.dp, 0.dp, 3.dp),
            )
            Text(
                text = getTransferDescription(transfer.state),
                style = MaterialTheme.typography.bodySmall,
                overflow = TextOverflow.Visible,
            )
        }

        if (transfer.canOpen) {
            TextButton(onClick = onOpen) {
                Text(stringResource(R.string.transfers_transfer_open_button).uppercaseCurrentLocale())
            }
        }

        if (transfer.canRetry) {
            TextButton(onClick = onRetry) {
                Text(stringResource(R.string.transfers_transfer_retry_button).uppercaseCurrentLocale())
            }
        }

        when (transfer.state) {
            is TransferState.Done -> IconButton(onClick = onAbort) {
                Icon(
                    Icons.Filled.Close,
                    stringResource(R.string.transfers_transfer_hide_button_content_desc),
                )
            }

            else -> TextButton(onClick = onAbort) {
                Text(
                    stringResource(R.string.transfers_transfer_cancel_button).uppercaseCurrentLocale(),
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

@Composable
fun getTransferDescription(state: TransferState): String {
    return when (state) {
        is TransferState.Waiting -> stringResource(R.string.transfers_transfer_state_waiting)
        is TransferState.Processing -> stringResource(R.string.transfers_transfer_state_processing)
        is TransferState.Transferring -> stringResource(R.string.transfers_transfer_state_transferring)
        is TransferState.Failed -> stringResource(
            R.string.transfers_transfer_state_failed,
            state.error,
        )
        is TransferState.Done -> stringResource(R.string.transfers_transfer_state_done)
    }
}

@Preview(showBackground = true)
@Composable
fun TransferRowPreview() {
    VaultTheme {
        Column {
            TransferRow(
                PreviewsData.transfersList[0],
                ImageBitmap(90, 110),
                onRetry = {},
                onAbort = {},
                onOpen = {},
            )

            TransferRow(
                PreviewsData.transfersList[1],
                ImageBitmap(90, 110),
                onRetry = {},
                onAbort = {},
                onOpen = {},
            )

            TransferRow(
                PreviewsData.transfersList[2],
                ImageBitmap(90, 110),
                onRetry = {},
                onAbort = {},
                onOpen = {},
            )

            TransferRow(
                PreviewsData.transfersList[3],
                ImageBitmap(90, 110),
                onRetry = {},
                onAbort = {},
                onOpen = {},
            )
        }
    }
}
