import SwiftUI
import VaultMobile

public struct RepoFileRow: View {
    public let container: Container
    public var file: RepoFile

    public var body: some View {
        FileRow(
            mobileVault: container.mobileVault,
            fileIcon: {
                FileIcon(fileIconCache: container.fileIconCache, attrs: file.fileIconAttrs)
            },
            name: file.name,
            sizeDisplay: file.sizeDisplay,
            modified: file.modified,
            isError: file.nameError != nil
        )
    }
}
