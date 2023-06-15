import SwiftUI
import VaultMobile

public let fileRowHeight = 45.0
public let fileRowInsetTop = 11.0
public let fileRowInsetBottom = 11.0
public let fileRowInsetLeading = 20.0
public let fileRowInsetTrailing = 20.0

public struct FileRow<FileIcon>: View where FileIcon: View {
    @ViewBuilder public let fileIcon: () -> FileIcon
    public let name: String
    public let sizeDisplay: String?
    public let modified: Int64?
    public let isError: Bool

    @ObservedObject var modifiedRelativeTime: RelativeTimeHelper

    public init(
        mobileVault: MobileVault, @ViewBuilder fileIcon: @escaping () -> FileIcon, name: String,
        sizeDisplay: String?, modified: Int64?, isError: Bool
    ) {
        self.fileIcon = fileIcon
        self.name = name
        self.sizeDisplay = sizeDisplay
        self.modified = modified
        self.isError = isError

        self.modifiedRelativeTime = RelativeTimeHelper(mobileVault: mobileVault, value: modified)
    }

    public var body: some View {
        HStack {
            fileIcon().padding(.trailing, 7)

            VStack(alignment: .leading, spacing: 0) {
                Text(isError ? "\(name) (ERROR)" : name)
                    .frame(height: 30)
                    .truncationMode(.middle)
                    .font(.system(.body))
                    .foregroundColor(isError ? Color(.systemRed) : Color(.label))

                let secondLineDisplay = secondLineDisplay()

                if !secondLineDisplay.isEmpty {
                    Text(secondLineDisplay)
                        .font(.system(.footnote))
                        .foregroundColor(Color(.secondaryLabel))
                }
            }

            Spacer()
        }
        .frame(height: fileRowHeight)
        .task {
            await modifiedRelativeTime.updateLoop()
        }
    }

    private func secondLineDisplay() -> String {
        var s = ""

        if let sizeDisplay = sizeDisplay {
            s += sizeDisplay
        }

        if sizeDisplay != nil && modifiedRelativeTime.display != nil {
            s += ", "
        }

        if let display = modifiedRelativeTime.display {
            s += display
        }

        return s
    }
}

public struct FileRowFilePreview: View {
    @StateObject var container = Container()

    public var body: some View {
        FileRow(
            mobileVault: container.mobileVault,
            fileIcon: {
                FileIcon(
                    fileIconCache: container.fileIconCache,
                    attrs: FileIconAttrs(
                        category: .image, isDl: false, isUl: false, isExport: false,
                        isImport: false, isAndroid: false, isIos: false, isVaultRepo: false,
                        isError: false),
                    size: .sm
                )
            },
            name: "example example example example example example example example example.jpg",
            sizeDisplay: "128.1 KB",
            modified: nowMs() - 2 * 60 * 1000,
            isError: false
        )
    }
}

public struct FileRowFolderPreview: View {
    @StateObject var container = Container()

    public var body: some View {
        FileRow(
            mobileVault: container.mobileVault,
            fileIcon: {
                FileIcon(
                    fileIconCache: container.fileIconCache,
                    attrs: FileIconAttrs(
                        category: .folder, isDl: false, isUl: false, isExport: false,
                        isImport: false, isAndroid: false, isIos: false, isVaultRepo: false,
                        isError: true),
                    size: .sm
                )
            },
            name: "example folder",
            sizeDisplay: nil,
            modified: nil,
            isError: true
        )
    }
}

public struct FileRow_Previews: PreviewProvider {
    static public var previews: some View {
        Group {
            FileRowFilePreview().previewDisplayName("File")
            FileRowFolderPreview().previewDisplayName("Folder")
        }.previewLayout(.fixed(width: 300, height: 70))
    }
}
