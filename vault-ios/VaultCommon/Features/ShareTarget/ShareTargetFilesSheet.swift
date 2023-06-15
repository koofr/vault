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
                Text(vm.files.count == 1 ? "\(vm.files.count) item" : "\(vm.files.count) items")
            )
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        onClose()
                    } label: {
                        Text("Dismiss")
                    }
                }
            }
        }
    }
}
