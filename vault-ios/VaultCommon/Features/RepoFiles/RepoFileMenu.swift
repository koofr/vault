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
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.get_info.menu_item",
                        defaultValue: "Get Info",
                        bundle: #bundle,
                        comment: "Context menu item on a file row that opens the file info sheet."
                    )
                )
            } icon: {
                Image(systemName: "info.circle")
            }
        }

        Button {
            vm.container.mobileVault.repoFilesRenameFile(
                repoId: file.repoId, encryptedPath: file.encryptedPath)
        } label: {
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.rename.menu_item",
                        defaultValue: "Rename",
                        bundle: #bundle,
                        comment: "Context menu item on a file row that opens rename dialog."
                    )
                )
            } icon: {
                Image(systemName: "pencil")
            }
        }

        Divider()

        Button {
            vm.container.mobileVault.repoFilesMoveFile(
                repoId: file.repoId, encryptedPath: file.encryptedPath, mode: .copy)
        } label: {
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.copy.menu_item",
                        defaultValue: "Copy",
                        bundle: #bundle,
                        comment: "Context menu item on a file row that opens copy file sheet."
                    )
                )
            } icon: {
                Image(systemName: "doc.on.doc")
            }
        }

        Button {
            vm.container.mobileVault.repoFilesMoveFile(
                repoId: file.repoId, encryptedPath: file.encryptedPath, mode: .move)
        } label: {
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.move.menu_item",
                        defaultValue: "Move",
                        bundle: #bundle,
                        comment: "Context menu item on a file row that opens move file sheet."
                    )
                )
            } icon: {
                Image(systemName: "folder")
            }
        }

        Divider()

        Button {
            vm.container.downloadHelper.downloadRepoFile(file: file)
        } label: {
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.download.menu_item",
                        defaultValue: "Download",
                        bundle: #bundle,
                        comment: "Context menu item on a file row that downloads the file."
                    )
                )
            } icon: {
                Image(systemName: "arrow.down.to.line.compact")
            }
        }

        Divider()

        Button(role: .destructive) {
            vm.container.mobileVault.repoFilesDeleteFile(
                repoId: file.repoId, encryptedPath: file.encryptedPath)
        } label: {
            Label {
                Text(
                    LocalizedStringResource(
                        "ios.repo_file_menu.delete.menu_item",
                        defaultValue: "Delete",
                        bundle: #bundle,
                        comment:
                            "Destructive context menu item on a file row that deletes the file."
                    )
                )
            } icon: {
                Image(systemName: "trash")
            }
        }
    }
}
