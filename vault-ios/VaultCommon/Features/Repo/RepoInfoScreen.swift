import SwiftUI
import VaultMobile

public class RepoInfoScreenViewModel: ObservableObject {
    public let container: Container
    public let navController: MainNavController
    public let repoId: String

    @Published public var biometricUnlockEnabled = false

    public init(container: Container, navController: MainNavController, repoId: String) {
        self.container = container
        self.navController = navController
        self.repoId = repoId

        self._biometricUnlockEnabled = Published(initialValue: checkBiometricUnlockEnabled())
    }

    public func updateBiometricUnlockEnabled() {
        self.biometricUnlockEnabled = checkBiometricUnlockEnabled()
    }

    public func checkBiometricUnlockEnabled() -> Bool {
        do {
            return try container.keychainRepoPasswordStorage.hasPassword(repoId: repoId)
        } catch {
            print("RepoInfoScreen keychainRepoPasswordStorage.hasPassword error: \(error)")

            return false
        }
    }

    public func disableBiometricUnlock() {
        do {
            try container.keychainRepoPasswordStorage.removePassword(repoId: repoId)

            updateBiometricUnlockEnabled()
        } catch {
            print("RepoInfoScreen keychainRepoPasswordStorage.removePassword error: \(error)")
        }
    }
}

public struct RepoInfoScreen: View {
    @ObservedObject var vm: RepoInfoScreenViewModel

    @ObservedObject private var info: Subscription<RepoInfo>

    public init(vm: RepoInfoScreenViewModel) {
        self.vm = vm

        self.info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.reposRepoSubscribe(repoId: vm.repoId, cb: cb)
            },
            getData: { v, id in
                v.reposRepoData(id: id)
            })
    }

    public var body: some View {
        if let repo = info.data?.repo {
            let unlocked = Binding(
                get: { repo.state == .unlocked },
                set: { value in
                    if value {
                        vm.container.sheets.show(
                            name: "repoInfoUnlock",
                            viewModel: RepoUnlockScreenViewModel(
                                container: vm.container, repoId: repo.id,
                                setupBiometricUnlockVisible: false)
                        ) { vm, hide in
                            RepoUnlockSheet(vm: vm, onDismiss: hide)
                        }
                    } else {
                        vm.container.mobileVault.reposLockRepo(repoId: vm.repoId)
                    }
                })

            let biometricUnlockEnabled = Binding(
                get: {
                    vm.biometricUnlockEnabled
                },
                set: { value in
                    if value {
                        vm.container.sheets.show(
                            viewModel: RepoSetupBiometricUnlockSheetViewModel(
                                container: vm.container, repoId: vm.repoId),
                            onHide: {
                                vm.updateBiometricUnlockEnabled()
                            },
                            content: { setupVm, hide in
                                RepoSetupBiometricUnlockSheet(vm: setupVm, onDismiss: hide)
                            })
                    } else {
                        vm.disableBiometricUnlock()
                    }
                })

            List {
                Section {
                    HStack {
                        Toggle(isOn: unlocked) {
                            VStack(alignment: .leading) {
                                Text("Unlocked").padding(.bottom, 0.5)
                                Text("Unlock or lock the Safe Box").font(.system(.footnote))
                                    .foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)

                    HStack {
                        Toggle(isOn: biometricUnlockEnabled) {
                            VStack(alignment: .leading) {
                                Text("Biometric unlock").padding(.bottom, 0.5)
                                Text("Use biometrics to unlock the Safe Box").font(
                                    .system(.footnote)
                                ).foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)

                    HStack {
                        Button {
                            vm.navController.push(.repoRemove(repoId: vm.repoId))
                        } label: {
                            VStack(alignment: .leading) {
                                Text("Destroy Safe Box…").padding(.bottom, 0.5).foregroundColor(
                                    Color(.label))
                                Text("Verify Safe Key and destroy the Safe box").font(
                                    .system(.footnote)
                                ).foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)
                }
            }
            .navigationTitle(repo.name)
        }
    }
}
