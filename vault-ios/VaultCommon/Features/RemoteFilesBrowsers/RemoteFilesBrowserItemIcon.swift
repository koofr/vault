import SwiftUI
import VaultMobile

public struct RemoteFilesBrowserItemIcon: View {
    public let fileIconCache: FileIconCache
    public let itemType: RemoteFilesBrowserItemType

    public var body: some View {
        VStack {
            switch itemType {
            case .bookmarks:
                Image("bookmarks").resizable().frame(width: 25, height: 25)
            case .place(origin: .hosted):
                Image("hosted").resizable().frame(width: 25, height: 25)
            case .place(origin: .desktop):
                Image("desktop").resizable().frame(width: 25, height: 25)
            case .place(origin: .dropbox):
                Image("dropbox").resizable().frame(width: 25, height: 25)
            case .place(origin: .googledrive):
                Image("googledrive").resizable().frame(width: 25, height: 25)
            case .place(origin: .onedrive):
                Image("onedrive").resizable().frame(width: 25, height: 25)
            case .place(origin: .share):
                Image("shared").resizable().frame(width: 25, height: 25)
            case .place(origin: .other):
                Image("hosted").resizable().frame(width: 25, height: 25)
            case .file(_, let fileIconAttrs):
                FileIcon(fileIconCache: fileIconCache, attrs: fileIconAttrs)
            case .shared:
                Image("shared").resizable().frame(width: 25, height: 25)
            }
        }
        .frame(width: 29, height: 29)
    }
}

struct RemoteFilesBrowserItemIcon_Previews: PreviewProvider {
    static var previews: some View {
        Group {
            Group {
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .bookmarks
                ).previewDisplayName("Bookmarks")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .hosted)
                ).previewDisplayName("Place Hosted")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .desktop)
                ).previewDisplayName("Place Desktop")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .dropbox)
                ).previewDisplayName("Place Dropbox")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .googledrive)
                ).previewDisplayName("Place Google Drive")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .onedrive)
                ).previewDisplayName("Place OneDrive")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .share)
                ).previewDisplayName("Place Share")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .place(origin: .other)
                ).previewDisplayName("Place Other")
            }
            Group {
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache,
                    itemType: .file(
                        typ: .dir,
                        fileIconAttrs: FileIconAttrs(
                            category: .folder,
                            isDl: false,
                            isUl: false,
                            isExport: false,
                            isImport: false,
                            isAndroid: false,
                            isIos: false,
                            isVaultRepo: false,
                            isError: false
                        ))
                ).previewDisplayName("Folder")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache,
                    itemType: .file(
                        typ: .file,
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
                        ))
                ).previewDisplayName("File")
                RemoteFilesBrowserItemIcon(
                    fileIconCache: Container().fileIconCache, itemType: .shared
                ).previewDisplayName("Shared")
            }
        }.previewLayout(.sizeThatFits)
    }
}
