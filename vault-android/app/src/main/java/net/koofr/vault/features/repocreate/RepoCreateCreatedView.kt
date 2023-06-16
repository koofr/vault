package net.koofr.vault.features.repocreate

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.koofr.vault.RepoCreated
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.features.repo.RepoConfigInfo

@Composable
fun RepoCreateCreatedView(vm: RepoCreateViewModel, created: RepoCreated, modifier: Modifier) {
    val navController = LocalNavController.current

    LazyColumn(
        modifier = modifier.fillMaxWidth(),
    ) {
        item {
            Column(modifier = Modifier.padding(17.dp)) {
                Text(
                    "Your Safe Box has been created.",
                    style = MaterialTheme.typography.displaySmall,
                )
                Spacer(modifier = Modifier.height(20.dp))

                Text(
                    "Before you start using your Safe Box please safely store the configuration.",
                    style = MaterialTheme.typography.bodyLarge,
                )
                Spacer(modifier = Modifier.height(20.dp))

                RepoConfigInfo(config = created.config, onSave = {
                    vm.saveConfig()
                })
                Spacer(modifier = Modifier.height(20.dp))

                Row(
                    horizontalArrangement = Arrangement.Center,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Button(onClick = {
                        navController.popBackStack("repos", false)
                        navController.navigate("repos/${created.repoId}/files")
                    }, enabled = vm.configSaved.value) {
                        Text("Continue")
                    }
                }
                Spacer(modifier = Modifier.height(40.dp))
            }
        }
    }
}
