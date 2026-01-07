package net.koofr.vault.features.repofilesdetails

import android.content.Context
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun RepoFilesDetailsNavMenu(vm: RepoFilesDetailsScreenViewModel, context: Context) {
    vm.content.value.let {
        when (it) {
            is RepoFilesDetailsScreenContent.Downloaded -> {
                DropdownMenuItem(text = {
                    Text("Share")
                }, onClick = {
                    vm.menuExpanded.value = false
                    vm.share(context, it.localFile, it.repoFile.contentType)
                })
            }

            else -> {}
        }
    }

    DropdownMenuItem(text = {
        Text("Rename")
    }, onClick = {
        vm.menuExpanded.value = false
        vm.rename()
    })

    DropdownMenuItem(text = {
        Text("Delete")
    }, onClick = {
        vm.menuExpanded.value = false
        vm.delete()
    })
}
