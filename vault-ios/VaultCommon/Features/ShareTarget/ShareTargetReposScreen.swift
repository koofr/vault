import SwiftUI
import VaultMobile

public struct ShareTargetReposScreen: View {
    private let vm: ShareTargetViewModel

    @ObservedObject private var repos: Subscription<Repos>

    public init(vm: ShareTargetViewModel) {
        self.vm = vm

        self.repos = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.reposSubscribe(cb: cb)
            },
            getData: { v, id in
                v.reposData(id: id)
            })
    }

    public var body: some View {
        Group {
            if let data = repos.data {
                RefreshableList(
                    status: data.status, isEmpty: data.repos.isEmpty,
                    onRefresh: {
                        vm.container.mobileVault.reposLoad()
                    },
                    empty: {
                        HStack {
                            Spacer()

                            VStack {
                                Text("No Safe Boxes yet").font(.largeTitle).padding(.bottom, 20)

                                Text("Open Koofr Vault app and create one.")
                                    .multilineTextAlignment(.center)
                            }

                            Spacer()
                        }
                    }
                ) {
                    List(data.repos, id: \.id) { repo in
                        ShareTargetReposRepoRow(vm: vm, repo: repo)
                    }
                }
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle("Save to Koofr Vault")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button {
                    vm.cancel()
                } label: {
                    Text("Cancel")
                }
            }

            ToolbarItem(placement: .bottomBar) {
                ShareTargetBottomBar(vm: vm)
            }
        }
    }
}

struct ShareTargetReposRepoRow: View {
    let vm: ShareTargetViewModel
    var repo: Repo

    var body: some View {
        HStack {
            Button {
                vm.navController.push(.repoFiles(repoId: repo.id, encryptedPath: "/"))
            } label: {
                RepoRow(repo: repo)
            }
        }
    }
}
