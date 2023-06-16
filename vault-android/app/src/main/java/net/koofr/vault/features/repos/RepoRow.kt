package net.koofr.vault.features.repos

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.koofr.vault.PreviewsData
import net.koofr.vault.R
import net.koofr.vault.Repo
import net.koofr.vault.RepoState
import net.koofr.vault.ui.theme.VaultTheme

@Composable
fun RepoRow(repo: Repo, onClick: () -> Unit, onMoreClick: (() -> Unit)? = null) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .clickable(
                onClick = onClick,
            )
            .height(60.dp)
            .fillMaxWidth()
            .semantics {
                contentDescription = "Safe Box ${repo.name}"
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
            if (repo.state == RepoState.LOCKED) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_locked),
                    contentDescription = "Locked",
                    tint = Color.Unspecified,
                    modifier = Modifier
                        .padding(10.dp)
                        .fillMaxSize(),
                )
            } else {
                Icon(
                    painter = painterResource(id = R.drawable.ic_unlocked),
                    contentDescription = "Unlocked",
                    tint = Color.Unspecified,
                    modifier = Modifier
                        .padding(10.dp)
                        .fillMaxSize(),
                )
            }
        }
        Text(
            text = repo.name,
            style = MaterialTheme.typography.bodyLarge,
            modifier = Modifier
                .padding(0.dp, 0.dp, 10.dp, 0.dp)
                .weight(1.0f),
        )
        onMoreClick?.let {
            IconButton(onClick = it) {
                Icon(Icons.Filled.MoreVert, "Info")
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun RepoRowPreview() {
    VaultTheme {
        Column {
            RepoRow(PreviewsData.repos[0], onClick = {}, onMoreClick = {})
            RepoRow(PreviewsData.repos[1], onClick = {})
        }
    }
}
