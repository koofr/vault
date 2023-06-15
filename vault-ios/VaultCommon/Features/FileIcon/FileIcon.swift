import SwiftUI
import VaultMobile

public struct FileIcon: View {
    public let fileIconCache: FileIconCache
    public let attrs: FileIconAttrs
    public let size: FileIconSize
    public let scale: Int
    public let height: CGFloat?

    init(
        fileIconCache: FileIconCache, attrs: FileIconAttrs, size: FileIconSize = .sm,
        scale: Int = 3, height: CGFloat? = nil
    ) {
        self.fileIconCache = fileIconCache
        self.attrs = attrs
        self.size = size
        self.scale = scale
        self.height = height
    }

    public var body: some View {
        Image(
            uiImage: fileIconCache.getIcon(
                props: FileIconProps(
                    size: size,
                    attrs: attrs),
                scale: scale
            )
        )
        .if(height != nil) { view in
            view
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(height: height!)
        }
    }
}

public struct FileIcon_Previews: PreviewProvider {
    static public var previews: some View {
        FileIcon(
            fileIconCache: Container().fileIconCache,
            attrs: FileIconAttrs(
                category: .generic, isDl: false, isUl: false, isExport: false, isImport: false,
                isAndroid: false, isIos: false, isVaultRepo: false, isError: false)
        )
    }
}
