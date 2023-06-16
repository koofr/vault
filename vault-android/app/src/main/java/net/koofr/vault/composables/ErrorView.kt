package net.koofr.vault.composables

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

@Composable
fun ErrorView(errorText: String, onRetry: (() -> Unit)?, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Column(
            modifier = Modifier
                .padding(17.dp)
                .fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                "Error",
                style = MaterialTheme.typography.displayMedium,
            )

            Spacer(modifier = Modifier.height(20.dp))

            Text(
                errorText,
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
            )

            onRetry?.let {
                Spacer(modifier = Modifier.height(20.dp))
                Button(onClick = onRetry) {
                    Text("Try again")
                }
            }
        }
    }
}
