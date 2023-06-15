import SwiftUI
import VaultMobile

public class ShareTargetRepoFilesScreenViewModel: ObservableObject {
    public let container: Container
    @Published public var shareTargetVm: ShareTargetViewModel
    public let repoId: String
    public let path: String

    public let browserId: UInt32

    @Published public var navController: ShareTargetNavController

    public init(
        container: Container, shareTargetVm: ShareTargetViewModel, repoId: String, path: String
    ) {
        self.container = container
        self.shareTargetVm = shareTargetVm
        self.repoId = repoId
        self.path = path

        browserId = container.mobileVault.repoFilesBrowsersCreate(
            repoId: repoId, path: path, options: RepoFilesBrowserOptions(selectName: nil))

        navController = shareTargetVm.navController
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

        info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesBrowsersInfoSubscribe(browserId: vm.browserId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesBrowsersInfoData(id: id)
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
                            cb: RepoFilesBrowserDirCreatedFn({ path in
                                vm.navController.push(.repoFiles(repoId: vm.repoId, path: path))
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
                        vm.shareTargetVm.upload(repoId: vm.repoId, path: vm.path)
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
