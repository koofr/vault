import SwiftUI

public struct ShareTargetFilesSheet: View {
    @ObservedObject public var vm: ShareTargetViewModel
    public let onClose: () -> Void

    public var body: some View {
        NavigationView {
            List(vm.files, id: \.localFile.id) { file in
                FileRow(
                    mobileVault: vm.container.mobileVault,
                    fileIcon: {
                        FileIcon(
                            fileIconCache: vm.container.fileIconCache,
                            attrs: file.localFile.fileIconAttrs)
                    },
                    name: file.localFile.name,
                    sizeDisplay: file.localFile.sizeDisplay,
                    modified: file.localFile.modified,
                    isError: false
                )
            }
            .listStyle(.plain)
            .navigationTitle(
                Text(
                    LocalizedStringResource(
                        "ios.share_target_files.title",
                        defaultValue: "\(vm.files.count) items",
                        bundle: #bundle,
                        comment:
                            "Navigation title in the share extension file list sheet showing number of selected items."
                    )
                )
            )
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        onClose()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.share_target_files.dismiss.button",
                                defaultValue: "Dismiss",
                                bundle: #bundle,
                                comment:
                                    "Toolbar button that closes the share extension file list sheet."
                            )
                        )
                    }
                }
            }
        }
    }
}
