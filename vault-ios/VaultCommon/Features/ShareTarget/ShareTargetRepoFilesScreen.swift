import SwiftUI
import VaultMobile

public class ShareTargetRepoFilesScreenViewModel: ObservableObject, WithRepoGuardViewModel {
    public let container: Container
    @Published public var shareTargetVm: ShareTargetViewModel
    public let repoId: String
    public let encryptedPath: String

    public let browserId: UInt32

    @Published public var navController: ShareTargetNavController

    @Published public var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    @Published public var repoGuardViewModel: RepoGuardViewModel

    public init(
        container: Container, shareTargetVm: ShareTargetViewModel, repoId: String,
        encryptedPath: String
    ) {
        self.container = container
        self.shareTargetVm = shareTargetVm
        self.repoId = repoId
        self.encryptedPath = encryptedPath

        let browserId = container.mobileVault.repoFilesBrowsersCreate(
            repoId: repoId, encryptedPath: encryptedPath,
            options: RepoFilesBrowserOptions(selectName: nil))

        self.browserId = browserId

        navController = shareTargetVm.navController

        info = VaultMobile.Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesBrowsersInfoSubscribe(browserId: browserId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesBrowsersInfoData(id: id)
            })

        repoGuardViewModel = RepoGuardViewModel(
            container: container, repoId: repoId, setupBiometricUnlockVisible: false)

        info.setOnData { [weak self] data in
            if let self = self {
                if let info = data {
                    self.repoGuardViewModel.update(
                        repoStatus: info.repoStatus, isLocked: info.isLocked)
                }
            }
        }
    }

    deinit {
        container.mobileVault.repoFilesBrowsersDestroy(browserId: browserId)
    }
}

public struct ShareTargetRepoFilesScreen: View {
    @ObservedObject var vm: ShareTargetRepoFilesScreenViewModel

    @ObservedObject var navController: ShareTargetNavController

    @ObservedObject private var info: Subscription<RepoFilesBrowserInfo>

    public init(vm: ShareTargetRepoFilesScreenViewModel) {
        self.vm = vm
        self.navController = vm.navController
        self.info = vm.info
    }

    public var body: some View {
        Group {
            if let info = info.data {
                RefreshableList(
                    status: info.status, isEmpty: info.items.isEmpty,
                    onRefresh: {
                        vm.container.mobileVault.repoFilesBrowsersLoadFiles(browserId: vm.browserId)
                    },
                    empty: {
                        EmptyFolderView()
                    }
                ) {
                    List(info.items, id: \.file.id) { item in
                        ShareTargetRepoFilesListRow(vm: vm, item: item)
                    }
                }
                .listStyle(.plain)
            }
        }
        .navigationTitle(info.data?.title ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button {
                        vm.container.mobileVault.repoFilesBrowsersCreateDir(
                            browserId: vm.browserId,
                            cb: RepoFilesBrowserDirCreatedFn({ encryptedPath in
                                vm.navController.push(
                                    .repoFiles(repoId: vm.repoId, encryptedPath: encryptedPath))
                            }))
                    } label: {
                        Label("New folder", systemImage: "folder.badge.plus")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }

            ToolbarItem(placement: .confirmationAction) {
                Button(
                    action: {
                        vm.shareTargetVm.upload(repoId: vm.repoId, encryptedPath: vm.encryptedPath)
                    },
                    label: {
                        Text("Upload")
                    }
                )
                .disabled(navController.state.isNavigating)
            }
        }
        .toolbar {
            ToolbarItem(placement: .bottomBar) {
                ShareTargetBottomBar(vm: vm.shareTargetVm)
            }
        }
    }
}
