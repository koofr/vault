package net.koofr.vault.features.repofilesdetails

import android.content.Context
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.koofr.vault.R
import net.koofr.vault.features.mobilevault.subscribe

@Composable
fun RepoFilesDetailsNavMenu(vm: RepoFilesDetailsScreenViewModel, context: Context) {
    val info = vm.info.data.value

    if (info != null && RepoFilesDetailsScreenContentData.isTextEditor(info.fileCategory)) {
        DropdownMenuItem(text = {
            Text(stringResource(R.string.repo_files_details_edit_menu_item))
        }, onClick = {
            vm.menuExpanded.value = false
            vm.edit()
        })
    }

    vm.content.value.let {
        when (it) {
            is RepoFilesDetailsScreenContent.Downloaded -> {
                DropdownMenuItem(text = {
                    Text(stringResource(R.string.repo_files_details_share_menu_item))
                }, onClick = {
                    vm.menuExpanded.value = false
                    vm.share(context, it.localFile, it.repoFile.contentType)
                })
            }

            is RepoFilesDetailsScreenContent.TextEditor -> {
                RepoFilesDetailsNavMenuShareTextEditor(vm, context)
            }

            else -> {}
        }
    }

    DropdownMenuItem(text = {
        Text(stringResource(R.string.repo_files_details_rename_menu_item))
    }, onClick = {
        vm.menuExpanded.value = false
        vm.rename()
    })

    DropdownMenuItem(text = {
        Text(stringResource(R.string.repo_files_details_delete_menu_item))
    }, onClick = {
        vm.menuExpanded.value = false
        vm.delete()
    })
}

@Composable
fun RepoFilesDetailsNavMenuShareTextEditor(vm: RepoFilesDetailsScreenViewModel, context: Context) {
    val content = subscribe(
        { v, cb -> v.repoFilesDetailsContentBytesSubscribe(detailsId = vm.detailsId, cb = cb) },
        { v, id -> v.repoFilesDetailsContentBytesData(id = id) },
    )

    DropdownMenuItem(text = {
        Text(stringResource(R.string.repo_files_details_share_menu_item))
    }, onClick = {
        vm.menuExpanded.value = false
        vm.shareText(context, content.value?.toString(Charsets.UTF_8) ?: "")
    })
}
