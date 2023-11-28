import SwiftUI
import VaultMobile

public class RepoFilesMoveScreenViewModel: ObservableObject, WithRepoGuardViewModel {
    public let container: Container
    @Published public var navController: RepoFilesMoveNavController
    public let repoId: String
    public let encryptedPath: String
    public let browserId: UInt32

    @Published public var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    @Published public var repoGuardViewModel: RepoGuardViewModel

    public init(
        container: Container, navController: RepoFilesMoveNavController, repoId: String,
        encryptedPath: String
    ) {
        self.container = container
        self.navController = navController
        self.repoId = repoId
        self.encryptedPath = encryptedPath

        let browserId = container.mobileVault.repoFilesBrowsersCreate(
            repoId: repoId, encryptedPath: encryptedPath,
            options: RepoFilesBrowserOptions(selectName: nil))

        self.browserId = browserId

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

public struct RepoFilesMoveScreen: View {
    @ObservedObject var vm: RepoFilesMoveScreenViewModel

    @ObservedObject public var navController: RepoFilesMoveNavController

    @ObservedObject private var info: Subscription<RepoFilesBrowserInfo>
    @ObservedObject private var moveInfo: Subscription<RepoFilesMoveInfo>

    public init(vm: RepoFilesMoveScreenViewModel) {
        self.vm = vm

        self.navController = vm.navController

        info = vm.info

        moveInfo = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesMoveInfoSubscribe(cb: cb)
            },
            getData: { v, id in
                v.repoFilesMoveInfoData(id: id)
            })
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
                        RepoFilesMoveListRow(vm: vm, item: item)
                    }
                }
                .listStyle(.plain)
            }
        }
        .navigationTitle(info.data?.title ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if vm.encryptedPath == "/" {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        vm.container.mobileVault.repoFilesMoveCancel()
                    } label: {
                        Text("Cancel")
                    }
                }
            }

            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button {
                        vm.container.mobileVault.repoFilesBrowsersCreateDir(
                            browserId: vm.browserId,
                            cb: RepoFilesBrowserDirCreatedFn { encryptedPath in
                                vm.navController.push(
                                    .repoFiles(repoId: vm.repoId, encryptedPath: encryptedPath))
                            })
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
                        vm.container.mobileVault.repoFilesMoveMoveFiles()
                    },
                    label: {
                        switch moveInfo.data?.mode {
                        case .copy: Text("Copy")
                        default: Text("Move")
                        }
                    }
                )
                .disabled(moveInfo.data?.canMove != true || navController.state.isNavigating)
            }
        }
        .toolbar {
            ToolbarItem(placement: .bottomBar) {
                if let moveInfo = moveInfo.data {
                    if moveInfo.srcFilesCount == 1 {
                        Text("\(moveInfo.srcFilesCount) item")
                    } else {
                        Text("\(moveInfo.srcFilesCount) items")
                    }
                }
            }
        }
    }
}
