import Foundation
import VaultMobile

public class RepoFilesDetailsScreenViewModel: ObservableObject, WithRepoGuardViewModel {
    public let container: Container
    public let navController: MainNavController
    public let repoId: String
    public let encryptedPath: String

    public var detailsId: UInt32

    @Published public var content: RepoFilesDetailsScreenContent

    @Published public var info: Subscription<RepoFilesDetailsInfo>
    @Published public var file: Subscription<RepoFile>

    @Published public var repoGuardViewModel: RepoGuardViewModel

    @Published public var textEditorText: String

    private var shouldDestroyHandled: Bool

    public init(
        container: Container, navController: MainNavController, repoId: String,
        encryptedPath: String, isEditing: Bool
    ) {
        self.container = container
        self.navController = navController
        self.repoId = repoId
        self.encryptedPath = encryptedPath

        let detailsId = container.mobileVault.repoFilesDetailsCreate(
            repoId: repoId, encryptedPath: encryptedPath, isEditing: isEditing,
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

        textEditorText = ""

        shouldDestroyHandled = false

        self.info.setOnData { [weak self] data in
            if let self = self {
                if let info = data {
                    self.repoGuardViewModel.update(
                        repoStatus: info.repoStatus, isLocked: info.isLocked)

                    if info.shouldDestroy && !shouldDestroyHandled {
                        shouldDestroyHandled = true

                        if let repoId = info.repoId {
                            let routes = info.encryptedParentPathChain.map {
                                MainRoute.repoFiles(repoId: repoId, encryptedPath: $0)
                            }

                            DispatchQueue.main.async {
                                self.navController.replace(routes)
                            }
                        }
                    }
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

    public func retryLoad() {
        if let file = file.data {
            load(file)
        }
    }

    private func load(_ file: RepoFile) {
        if RepoFilesDetailsScreenContentData.isTextEditor(file) {
            content = .textEditor(file: file)

            return
        }

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
