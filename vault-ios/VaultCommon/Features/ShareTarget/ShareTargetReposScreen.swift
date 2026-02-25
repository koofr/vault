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
                                Text(
                                    LocalizedStringResource(
                                        "ios.share_target.repos.empty.label",
                                        defaultValue: "No Safe Boxes yet",
                                        bundle: #bundle,
                                        comment:
                                            "Empty-state headline in share extension when user has no Safe Boxes."
                                    )
                                )
                                .font(.largeTitle)
                                .padding(.bottom, 20)

                                Text(
                                    LocalizedStringResource(
                                        "ios.share_target.repos.open_app_create.label",
                                        defaultValue: "Open Koofr Vault app and create one.",
                                        bundle: #bundle,
                                        comment:
                                            "Empty-state instruction in share extension telling user to create a Safe Box in the main app."
                                    )
                                )
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
        .navigationTitle(
            Text(
                LocalizedStringResource(
                    "ios.share_target.repos.title",
                    defaultValue: "Save to Koofr Vault",
                    bundle: #bundle,
                    comment:
                        "Navigation title for the share extension screen where user chooses destination Safe Box."
                )
            )
        )
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button {
                    vm.cancel()
                } label: {
                    Text(
                        LocalizedStringResource(
                            "ios.share_target.repos.cancel.button",
                            defaultValue: "Cancel",
                            bundle: #bundle,
                            comment: "Toolbar button that cancels the share extension."
                        )
                    )
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
                vm.navController.navController.push(.repoFiles(repoId: repo.id, encryptedPath: "/"))
            } label: {
                RepoRow(repo: repo)
            }
        }
    }
}
