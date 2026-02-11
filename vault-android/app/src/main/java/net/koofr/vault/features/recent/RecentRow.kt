package net.koofr.vault.features.recent

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.R
import net.koofr.vault.ui.theme.VaultTheme

@Composable
fun RecentRow(repoName: String, onClick: () -> Unit) {
    val context = LocalContext.current

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .clickable(
                onClick = onClick,
            )
            .height(60.dp)
            .fillMaxWidth()
            .semantics {
                contentDescription = context.getString(R.string.repos_recent_content_desc, repoName)
            },
    ) {
        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .width(60.dp)
                .height(60.dp)
                .padding(7.dp, 7.dp),
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_recent),
                contentDescription = stringResource(R.string.repos_recent_label),
                tint = Color.Unspecified,
                modifier = Modifier
                    .padding(10.dp)
                    .fillMaxSize(),
            )
        }
        Text(
            text = stringResource(R.string.repos_recent_label),
            style = MaterialTheme.typography.bodyLarge,
            modifier = Modifier
                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                .weight(1.0f),
        )
    }
}

@Preview(showBackground = true)
@Composable
fun RecentRowPreview() {
    VaultTheme {
        Column {
            RecentRow(repoName = "Vault", onClick = {})
        }
    }
}
