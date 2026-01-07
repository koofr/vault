package net.koofr.vault.features.repofilesdetails

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.koofr.vault.RepoFilesDetailsInfo
import net.koofr.vault.Status
import net.koofr.vault.features.relativetime.relativeTime

@Composable
fun RepoFilesDetailsEditorInfo(
    vm: RepoFilesDetailsScreenViewModel,
    info: RepoFilesDetailsInfo,
) {
    val isLoading = info.status is Status.Loading || info.contentStatus is Status.Loading
    val isSaving = info.saveStatus is Status.Loading

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 35.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        if (isLoading) {
            Text(
                "Loading...",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.secondary,
            )
        } else if (isSaving) {
            Text(
                "Saving...",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.secondary,
            )
        } else if (info.error != null) {
            Text(
                info.error!!,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .padding(horizontal = 16.dp)
                    .semantics {
                        contentDescription = "File error"
                    },
            )
        } else {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(15.dp),
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    Text(
                        "Changes are saved automatically.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.secondary,
                    )

                    info.fileModified?.let { modified ->
                        val relativeTime = relativeTime(vm.mobileVault, modified)
                        Text(
                            "Last saved $relativeTime",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                }

                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .background(
                            if (info.isDirty) Color(0xFFFFA500) else Color(0xFF4CAF50),
                            CircleShape,
                        )
                        .semantics {
                            contentDescription = if (info.isDirty) "File modified" else "File unchanged"
                        },
                )
            }
        }
    }
}
