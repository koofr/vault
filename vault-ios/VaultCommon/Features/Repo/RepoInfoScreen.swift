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

    @State private var autoLockAfterOptionsPresented = false

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

            let repoAutoLockOnAppHidden = Binding(
                get: {
                    repo.autoLock.onAppHidden
                },
                set: { value in
                    vm.container.mobileVault.reposSetAutoLock(
                        repoId: vm.repoId,
                        autoLock: RepoAutoLock(after: repo.autoLock.after, onAppHidden: value))
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

                    Button {
                        autoLockAfterOptionsPresented = true
                    } label: {
                        VStack(alignment: .leading) {
                            Text("Automatically lock after").padding(.bottom, 0.5)
                                .foregroundColor(Color(.label))

                            Text(repoAutoLockAfterDisplay(repo.autoLock.after)).font(
                                .system(.footnote)
                            ).foregroundColor(Color(.secondaryLabel))
                        }
                    }
                    .padding(.vertical, 2)
                    .confirmationDialog(
                        "Automatically lock after", isPresented: $autoLockAfterOptionsPresented
                    ) {
                        ForEach(
                            getRepoAutoLockAfterOptions(current: repo.autoLock.after), id: \.self
                        ) { option in
                            Button(repoAutoLockAfterDisplay(option)) {
                                vm.container.mobileVault.reposSetAutoLock(
                                    repoId: vm.repoId,
                                    autoLock: RepoAutoLock(
                                        after: option, onAppHidden: repo.autoLock.onAppHidden))
                            }
                        }
                    }

                    HStack {
                        Toggle(isOn: repoAutoLockOnAppHidden) {
                            VStack(alignment: .leading) {
                                Text("Lock when app hidden").padding(.bottom, 0.5)
                                Text("When switching apps or locking the screen").font(
                                    .system(.footnote)
                                ).foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)
                }

                Section {
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

    func repoAutoLockAfterDisplay(_ after: RepoAutoLockAfter) -> String {
        switch after {
        case .noLimit: return "No time limit"
        case .inactive1Minute: return "1 minute of inactivity"
        case .inactive5Mininutes: return "5 minutes of inactivity"
        case .inactive10Minutes: return "10 minutes of inactivity"
        case .inactive30Minutes: return "30 minutes of inactivity"
        case .inactive1Hour: return "1 hour of inactivity"
        case .inactive2Hours: return "2 hours of inactivity"
        case .inactive4Hours: return "4 hours of inactivity"
        case .custom(let seconds): return "Custom (\(seconds) seconds)"
        }
    }

    func getRepoAutoLockAfterOptions(current: RepoAutoLockAfter) -> [RepoAutoLockAfter] {
        var options: [RepoAutoLockAfter] = [
            .noLimit,
            .inactive1Minute,
            .inactive5Mininutes,
            .inactive10Minutes,
            .inactive30Minutes,
            .inactive1Hour,
            .inactive2Hours,
            .inactive4Hours,
        ]

        switch current {
        case .custom(_): options.append(current)
        default: ()
        }

        return options
    }
}
