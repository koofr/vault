package net.koofr.vault.features.repofilesdetails

import android.content.Context
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import net.koofr.vault.features.mobilevault.subscribe

@Composable
fun RepoFilesDetailsNavMenu(vm: RepoFilesDetailsScreenViewModel, context: Context) {
    val info = vm.info.data.value

    if (info != null && !info.isEditing && RepoFilesDetailsScreenContentData.isTextEditor(info.fileCategory)) {
        DropdownMenuItem(text = {
            Text("Edit")
        }, onClick = {
            vm.menuExpanded.value = false
            vm.edit()
        })
    }

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

            is RepoFilesDetailsScreenContent.TextEditor -> {
                RepoFilesDetailsTextEditorNavMenu(vm, context)
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

@Composable
fun RepoFilesDetailsTextEditorNavMenu(vm: RepoFilesDetailsScreenViewModel, context: Context) {
    val content = subscribe(
        { v, cb -> v.repoFilesDetailsContentBytesSubscribe(detailsId = vm.detailsId, cb = cb) },
        { v, id -> v.repoFilesDetailsContentBytesData(id = id) },
    )

    DropdownMenuItem(text = {
        Text("Share")
    }, onClick = {
        vm.menuExpanded.value = false
        vm.shareText(context, content.value?.toString(Charsets.UTF_8) ?: "")
    })
}
