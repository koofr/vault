import Foundation
import VaultMobile

public class ShareTargetViewModel: ObservableObject {
    public let container: Container
    @Published public var files: [ShareTargetFile]
    public let onUpload: () -> Void
    public let onCancel: () -> Void

    public let navController: ShareTargetNavController

    public init(
        container: Container, files: [UploadFile], onUpload: @escaping () -> Void,
        onCancel: @escaping () -> Void
    ) {
        self.container = container
        self.files = files.map { uploadFile in
            let localFile = container.mobileVault.localFilesFileInfo(
                name: uploadFile.name, typ: .file, size: uploadFile.size, modified: nil)

            return ShareTargetFile(localFile: localFile, uploadFile: uploadFile)
        }
        self.onUpload = onUpload
        self.onCancel = onCancel

        self.navController = NavController(rootRoute: .repos)
    }

    public func cancel() {
        onCancel()
    }

    public func upload(repoId: String, path: String) {
        container.uploadHelper.uploadFiles(
            repoId: repoId, parentPath: path, files: files.map { $0.uploadFile })

        onUpload()
    }
}
