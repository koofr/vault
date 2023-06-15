import SwiftUI
import VaultMobile

public struct RepoCreateScreen: View {
    @ObservedObject public var vm: RepoCreateScreenViewModel

    @ObservedObject private var info: Subscription<RepoCreateInfo>

    public init(vm: RepoCreateScreenViewModel) {
        self.vm = vm

        info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoCreateInfoSubscribe(createId: vm.createId, cb: cb)
            },
            getData: { v, id in
                v.repoCreateInfoData(id: id)
            })

        info.setOnData { data in
            if let info = data {
                switch info {
                case .form(let form):
                    if form.password != vm.password {
                        vm.password = form.password
                    }

                    let formSalt = form.salt ?? ""

                    if formSalt != vm.salt {
                        vm.salt = formSalt
                    }
                default: ()
                }
            }
        }
    }

    public var body: some View {
        VStack {
            if let info = info.data {
                switch info {
                case .form(let form):
                    switch form.createLoadStatus {
                    case .initial, .loading(_):
                        LoadingView()
                    case .loaded:
                        RepoCreateFormView(vm: vm, form: form)
                    case .err(let error, _):
                        ErrorView(
                            errorText: error,
                            onRetry: {
                                vm.container.mobileVault.repoCreateCreateLoad(createId: vm.createId)
                            })
                    }
                case .created(let created):
                    RepoCreateCreatedView(
                        config: created.config,
                        onContinue: {
                            vm.navController.pop()
                            vm.navController.push(.repoFiles(repoId: created.repoId, path: "/"))
                        })
                }
            }
        }
        .navigationTitle("Create a new Safe Box")
        .navigationBarTitleDisplayMode(.inline)
    }
}
