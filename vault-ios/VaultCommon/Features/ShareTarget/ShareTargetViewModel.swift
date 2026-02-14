import Foundation
import VaultMobile

public class ShareTargetViewModel: ObservableObject {
    public let container: Container
    @Published public var files: [ShareTargetFile]
    public let beforeUpload: () -> Void
    public let onUpload: () -> Void
    public let onCancel: () -> Void

    public let navController: ShareTargetVaultNavController

    public init(
        container: Container,
        files: [UploadFile],
        beforeUpload: @escaping () -> Void,
        onUpload: @escaping () -> Void,
        onCancel: @escaping () -> Void
    ) {
        self.container = container
        self.files = files.map { uploadFile in
            let localFile = container.mobileVault.localFilesFileInfo(
                name: uploadFile.name, typ: .file, size: uploadFile.size, modified: nil)

            return ShareTargetFile(localFile: localFile, uploadFile: uploadFile)
        }
        self.beforeUpload = beforeUpload
        self.onUpload = onUpload
        self.onCancel = onCancel

        self.navController = ShareTargetVaultNavController(
            navController: ShareTargetNavController(rootRoute: .repos),
            mobileVault: container.mobileVault)
    }

    public func cancel() {
        onCancel()
    }

    public func upload(repoId: String, encryptedPath: String) {
        beforeUpload()

        container.uploadHelper.uploadFiles(
            repoId: repoId, encryptedParentPath: encryptedPath, files: files.map { $0.uploadFile })

        onUpload()
    }
}
