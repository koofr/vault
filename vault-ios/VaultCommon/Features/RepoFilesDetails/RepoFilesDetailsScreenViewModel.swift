import Foundation
import VaultMobile

public class RepoFilesDetailsScreenViewModel: ObservableObject, WithRepoGuardViewModel {
    public let container: Container
    public let repoId: String
    public let encryptedPath: String

    public var detailsId: UInt32

    @Published public var content: RepoFilesDetailsScreenContent

    @Published public var info: Subscription<RepoFilesDetailsInfo>
    @Published public var file: Subscription<RepoFile>

    @Published public var repoGuardViewModel: RepoGuardViewModel

    public init(container: Container, repoId: String, encryptedPath: String) {
        self.container = container
        self.repoId = repoId
        self.encryptedPath = encryptedPath

        let detailsId = container.mobileVault.repoFilesDetailsCreate(
            repoId: repoId, encryptedPath: encryptedPath, isEditing: false,
            options: RepoFilesDetailsOptions(
                loadContent: FilesFilter(categories: [.code, .text], exts: []),
                autosaveIntervalMs: 20000))

        self.detailsId = detailsId

        self.content = .loading

        info = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsInfoSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsInfoData(id: id)
            })

        file = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsFileSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsFileData(id: id)
            })

        repoGuardViewModel = RepoGuardViewModel(
            container: container, repoId: repoId, setupBiometricUnlockVisible: true)

        self.info.setOnData { [weak self] data in
            if let self = self {
                if let info = data {
                    self.repoGuardViewModel.update(
                        repoStatus: info.repoStatus, isLocked: info.isLocked)
                }
            }
        }

        self.file.setOnData { [weak self] data in
            if let self = self {
                if let file = data {
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
