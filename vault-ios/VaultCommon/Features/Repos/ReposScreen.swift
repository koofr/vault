import SwiftUI
import VaultMobile

public enum ReposListItem: Hashable {
    case repo(repo: Repo)
    case create

    var id: String {
        switch self {
        case .repo(let repo):
            return repo.id
        case .create:
            return "create"
        }
    }
}

public enum RedirecToRepoCreate: Equatable {
    case loading
    case notNeeded
    case shouldRedirect
    case redirected
}

public class ReposScreenViewModel: ObservableObject {
    public let container: Container
    public let navController: MainNavController

    @Published public var items: [ReposListItem]
    @Published public var repos: Subscription<Repos>

    @Published public var appearCount: Int = 0

    @Published public var redirecToRepoCreate: RedirecToRepoCreate = .loading

    init(container: Container, navController: MainNavController) {
        self.container = container
        self.navController = navController

        items = [ReposListItem]()

        repos = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.reposSubscribe(cb: cb)
            },
            getData: { v, id in
                v.reposData(id: id)
            })

        repos.setOnData { [weak self] data in
            if let self = self {
                var items = [ReposListItem]()

                if let data = data {
                    if redirecToRepoCreate == .loading && data.status == .loaded {
                        if data.repos.isEmpty {
                            redirecToRepoCreate = .shouldRedirect
                        } else {
                            redirecToRepoCreate = .notNeeded
                        }
                    }

                    for repo in data.repos {
                        items.append(ReposListItem.repo(repo: repo))
                    }

                    if !data.repos.isEmpty {
                        items.append(.create)
                    }
                }

                self.items = items
            }
        }
    }

    func checkRedirectToCreate() {
        if redirecToRepoCreate == .shouldRedirect {
            redirecToRepoCreate = .redirected

            navController.push(.repoCreate)
        }
    }
}

public struct ReposScreen: View {
    @ObservedObject var vm: ReposScreenViewModel

    @ObservedObject public var repos: Subscription<Repos>

    public init(vm: ReposScreenViewModel) {
        self.vm = vm

        self.repos = vm.repos
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

                                Button {
                                    vm.navController.push(.repoCreate)
                                } label: {
                                    Text("Create your first one")
                                        .foregroundColor(Color(.link))
                                        .multilineTextAlignment(.center)
                                }
                            }

                            Spacer()
                        }
                    }
                ) {
                    FrozenList(vm.items, id: \.id) { item in
                        switch item {
                        case .repo(let repo):
                            ReposRepoRow(vm: vm, repo: repo)
                        case .create:
                            Button {
                                vm.navController.push(.repoCreate)
                            } label: {
                                RepoCreateRow()
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("Vault")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                TransfersButton(container: vm.container)
            }

            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    vm.container.sheets.show(
                        name: "settings",
                        content: { _, hide in
                            SettingsSheet(container: vm.container, onDismiss: hide)
                        })
                } label: {
                    Image(systemName: "gearshape")
                }
            }
        }
        .onAppear {
            if vm.appearCount > 0 {
                vm.container.mobileVault.load()
            }

            vm.appearCount += 1

            vm.checkRedirectToCreate()
        }
        .onChange(of: vm.redirecToRepoCreate) { _ in
            vm.checkRedirectToCreate()
        }
    }
}

struct ReposRepoRow: View {
    let vm: ReposScreenViewModel
    var repo: Repo

    var body: some View {
        HStack {
            Button {
                vm.navController.push(.repoFiles(repoId: repo.id, encryptedPath: "/"))
            } label: {
                RepoRow(repo: repo)
            }

            Button {
                // handled with onTapGesture, does not work correctly if handled
                // here
            } label: {
                Image(systemName: "info.circle").onTapGesture {
                    vm.navController.push(.repoInfo(repoId: repo.id))
                }
            }
        }
    }
}
