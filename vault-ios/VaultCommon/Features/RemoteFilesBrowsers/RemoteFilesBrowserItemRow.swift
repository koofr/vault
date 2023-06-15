import SwiftUI
import VaultMobile

public struct RemoteFilesBrowserItemRow: View {
    public let container: Container
    public var item: RemoteFilesBrowserItem

    public var body: some View {
        FileRow(
            mobileVault: container.mobileVault,
            fileIcon: {
                RemoteFilesBrowserItemIcon(
                    fileIconCache: container.fileIconCache, itemType: item.typ)
            },
            name: item.name,
            sizeDisplay: item.sizeDisplay,
            modified: item.modified,
            isError: false
        )
    }
}

struct RemoteFilesBrowserItemRow_Previews: PreviewProvider {
    static var previews: some View {
        Group {
            RemoteFilesBrowserItemRow(
                container: Container(),
                item: RemoteFilesBrowserItem(
                    id: "id", mountId: nil, path: nil, name: "Bookmarks", typ: .bookmarks,
                    sizeDisplay: nil, modified: nil, isSelected: false)
            ).previewDisplayName("Bookmarks")
            RemoteFilesBrowserItemRow(
                container: Container(),
                item: RemoteFilesBrowserItem(
                    id: "id", mountId: nil, path: nil, name: "Koofr", typ: .place(origin: .hosted),
                    sizeDisplay: nil, modified: nil, isSelected: false)
            ).previewDisplayName("Place Hosted")
            RemoteFilesBrowserItemRow(
                container: Container(),
                item: RemoteFilesBrowserItem(
                    id: "id", mountId: nil, path: nil, name: "File",
                    typ: .file(
                        typ: .dir,
                        fileIconAttrs: FileIconAttrs(
                            category: .generic,
                            isDl: false,
                            isUl: false,
                            isExport: false,
                            isImport: false,
                            isAndroid: false,
                            isIos: false,
                            isVaultRepo: false,
                            isError: false
                        )), sizeDisplay: "128.1 KB", modified: nowMs() - 2 * 60 * 1000,
                    isSelected: false)
            ).previewDisplayName("File")
        }.previewLayout(.sizeThatFits)
    }
}
