import Foundation
import VaultMobile

public class RepoFilesDetailsScreenViewModel: ObservableObject {
    public let container: Container
    public let repoId: String
    public let path: String

    public var detailsId: UInt32

    @Published public var content: RepoFilesDetailsScreenContent

    public var file: Subscription<RepoFile>

    public init(container: Container, repoId: String, path: String) {
        self.container = container
        self.repoId = repoId
        self.path = path

        let detailsId = container.mobileVault.repoFilesDetailsCreate(
            repoId: repoId, path: path, isEditing: false,
            options: RepoFilesDetailsOptions(
                loadContent: FilesFilter(categories: [.code, .text], exts: []),
                autosaveIntervalMs: 20000))

        self.detailsId = detailsId

        self.content = .loading

        self.file = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsFileSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsFileData(id: id)
            })

        self.file.setOnData { [weak self] file in
            if let self = self {
                if let file = file {
                    self.load(file)
                }
            }
        }
    }

    deinit {
        container.mobileVault.repoFilesDetailsDestroy(detailsId: detailsId)
    }

    public func load(_ file: RepoFile) {
        if let loader = RepoFilesDetailsScreenContentData.getLoader(
            file: file,
            onWarning: { warning in
                self.container.mobileVault.notificationsShow(message: warning)
            })
        {
            content = .downloading

            let localBasePath = container.storageHelper.getTempDir().path

            container.mobileVault.repoFilesDetailsDownloadTempFile(
                detailsId: detailsId,
                localBasePath: localBasePath,
                onDone: TransfersDownloadDoneFn { [weak self] localFilePath in
                    if let self = self {
                        let localFileURL = URL(fileURLWithPath: localFilePath)

                        Task { @MainActor in
                            let data = await loader(localFileURL)

                            self.content = .downloaded(localFileURL: localFileURL, data: data)
                        }
                    }
                }
            )
        } else {
            content = .notSupported(file: file)
        }
    }
}
