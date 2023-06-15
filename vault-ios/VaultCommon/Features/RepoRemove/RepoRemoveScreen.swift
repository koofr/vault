import SwiftUI
import VaultMobile

public class RepoRemoveScreenViewModel: ObservableObject {
    public let container: Container
    public let navController: MainNavController

    public let repoId: String

    public let removeId: UInt32

    @Published public var password: String = ""

    public init(container: Container, navController: MainNavController, repoId: String) {
        self.container = container
        self.navController = navController
        self.repoId = repoId

        removeId = container.mobileVault.repoRemoveCreate(repoId: repoId)
    }

    deinit {
        container.mobileVault.repoRemoveDestroy(removeId: removeId)
    }

    public func destroy() {
        container.mobileVault.repoRemoveRemove(
            removeId: removeId, password: password,
            cb: RepoRemovedFn { [weak self] in
                if let self = self {
                    self.navController.pop()
                    self.navController.pop()
                }
            })
    }
}

public struct RepoRemoveScreen: View {
    @ObservedObject var vm: RepoRemoveScreenViewModel

    @ObservedObject private var info: Subscription<RepoRemoveInfo>

    public var canRemove: Bool {
        if let info = info.data {
            switch info.status {
            case .loading(_): return false
            default: ()
            }
        }

        return !vm.password.isEmpty
    }

    public init(vm: RepoRemoveScreenViewModel) {
        self.vm = vm

        self.info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoRemoveInfoSubscribe(removeId: vm.removeId, cb: cb)
            },
            getData: { v, id in
                v.repoRemoveInfoData(id: id)
            })
    }

    public var body: some View {
        ScrollView {
            VStack(alignment: .leading) {
                if let info = info.data {
                    if let repoName = info.repoName {
                        Text("Do you really want to destroy Safe Box **\(repoName)**?").padding(
                            .bottom, 20)
                    }

                    Text(
                        "Destroying the Safe Box will keep all the files on Koofr but remove the configuration so you won't be able to decrypt the files if you didn't save the configuration."
                    ).padding(.bottom, 20)

                    Text("**This action cannot be undone.**").padding(.bottom, 20)

                    Text("Enter your Safe Key to confirm the removal:").padding(.bottom, 20)

                    switch info.status {
                    case .err(let error, _):
                        Text(error)
                            .font(.body)
                            .foregroundColor(Color(.systemRed))
                            .frame(alignment: .leading)
                            .padding(.bottom, 20)
                    default:
                        EmptyView()
                    }

                    RepoPasswordField(
                        text: $vm.password,
                        onSubmit: {
                            if canRemove {
                                vm.destroy()
                            }
                        },
                        autoFocus: true
                    )
                }
            }
            .padding()
        }
        .navigationTitle("Destroy Safe Box")
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button(
                    action: {
                        vm.destroy()
                    },
                    label: {
                        Text("Destroy")
                    }
                )
                .tint(Color(.systemRed))
                .disabled(!canRemove)
            }
        }
    }
}
