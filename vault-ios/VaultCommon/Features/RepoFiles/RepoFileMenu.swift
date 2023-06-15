import SwiftUI
import VaultMobile

public struct RepoFileMenu: View {
    public let vm: RepoFilesScreenViewModel
    public let file: RepoFile

    public var body: some View {
        Button {
            vm.container.sheets.show { _, hide in
                RepoFileInfoSheet(vm: vm, file: file, onDismiss: hide)
            }
        } label: {
            Label("Get Info", systemImage: "info.circle")
        }

        if file.path != nil {
            Button {
                if let path = file.path {
                    vm.container.mobileVault.repoFilesRenameFile(repoId: file.repoId, path: path)
                }
            } label: {
                Label("Rename", systemImage: "pencil")
            }

            Divider()

            Button {
                if let path = file.path {
                    vm.container.mobileVault.repoFilesMoveFile(
                        repoId: file.repoId, path: path, mode: .copy)
                }
            } label: {
                Label("Copy", systemImage: "doc.on.doc")
            }

            Button {
                if let path = file.path {
                    vm.container.mobileVault.repoFilesMoveFile(
                        repoId: file.repoId, path: path, mode: .move)
                }
            } label: {
                Label("Move", systemImage: "folder")
            }

            Divider()

            Button {
                vm.container.downloadHelper.downloadRepoFile(file: file)
            } label: {
                Label("Download", systemImage: "arrow.down.to.line.compact")
            }

            Divider()

            Button(role: .destructive) {
                if let path = file.path {
                    vm.container.mobileVault.repoFilesDeleteFile(repoId: file.repoId, path: path)
                }
            } label: {
                Label("Delete", systemImage: "trash")
            }
        }
    }
}
