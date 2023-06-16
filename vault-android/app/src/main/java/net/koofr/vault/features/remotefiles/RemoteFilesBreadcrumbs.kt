package net.koofr.vault.features.remotefiles

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import net.koofr.vault.RemoteFilesBreadcrumb

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun RemoteFilesBreadcrumbs(breadcrumbs: List<RemoteFilesBreadcrumb>) {
    FlowRow(verticalArrangement = Arrangement.Center) {
        breadcrumbs.forEach {
            key(it.id) {
                Text(it.name)

                if (!it.last) {
                    Icon(
                        imageVector = Icons.Filled.ChevronRight,
                        contentDescription = null,
                    )
                }
            }
        }
    }
}
