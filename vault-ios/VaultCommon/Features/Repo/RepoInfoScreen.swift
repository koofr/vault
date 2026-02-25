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
                                Text(
                                    repo.state == .unlocked
                                        ? LocalizedStringResource(
                                            "ios.repo_info.unlocked.label",
                                            defaultValue: "Unlocked",
                                            bundle: #bundle,
                                            comment:
                                                "Toggle label in Safe Box settings when the Safe Box is currently unlocked."
                                        )
                                        : LocalizedStringResource(
                                            "ios.repo_info.locked.label",
                                            defaultValue: "Locked",
                                            bundle: #bundle,
                                            comment:
                                                "Toggle label in Safe Box settings when the Safe Box is currently locked."
                                        )
                                )
                                .padding(.bottom, 0.5)
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.unlock.description",
                                        defaultValue: "Unlock or lock the Safe Box",
                                        bundle: #bundle,
                                        comment:
                                            "Description under the lock state toggle in Safe Box settings."
                                    )
                                )
                                .font(.system(.footnote))
                                .foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)

                    HStack {
                        Toggle(isOn: biometricUnlockEnabled) {
                            VStack(alignment: .leading) {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.biometric_unlock.label",
                                        defaultValue: "Biometric unlock",
                                        bundle: #bundle,
                                        comment:
                                            "Toggle label for enabling biometric unlock in Safe Box settings."
                                    )
                                )
                                .padding(.bottom, 0.5)
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.biometric_unlock.description",
                                        defaultValue: "Use biometrics to unlock the Safe Box",
                                        bundle: #bundle,
                                        comment:
                                            "Description under the biometric unlock toggle in Safe Box settings."
                                    )
                                )
                                .font(.system(.footnote))
                                .foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)

                    Button {
                        autoLockAfterOptionsPresented = true
                    } label: {
                        VStack(alignment: .leading) {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_info.auto_lock_after.label",
                                    defaultValue: "Automatically lock after",
                                    bundle: #bundle,
                                    comment:
                                        "Label for the row that opens auto-lock timeout options in Safe Box settings."
                                )
                            )
                            .padding(.bottom, 0.5)
                            .foregroundColor(Color(.label))

                            Text(repoAutoLockAfterDisplay(repo.autoLock.after))
                                .font(.system(.footnote))
                                .foregroundColor(Color(.secondaryLabel))
                        }
                    }
                    .padding(.vertical, 2)
                    .confirmationDialog(
                        Text(
                            LocalizedStringResource(
                                "ios.repo_info.auto_lock_after.label",
                                defaultValue: "Automatically lock after",
                                bundle: #bundle,
                                comment:
                                    "Title of the confirmation dialog for choosing auto-lock timeout."
                            )
                        ),
                        isPresented: $autoLockAfterOptionsPresented
                    ) {
                        ForEach(
                            getRepoAutoLockAfterOptions(current: repo.autoLock.after), id: \.self
                        ) { option in
                            Button {
                                vm.container.mobileVault.reposSetAutoLock(
                                    repoId: vm.repoId,
                                    autoLock: RepoAutoLock(
                                        after: option, onAppHidden: repo.autoLock.onAppHidden))
                            } label: {
                                Text(repoAutoLockAfterDisplay(option))
                            }
                        }
                    }

                    HStack {
                        Toggle(isOn: repoAutoLockOnAppHidden) {
                            VStack(alignment: .leading) {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.lock_when_app_hidden.label",
                                        defaultValue: "Lock when app hidden",
                                        bundle: #bundle,
                                        comment:
                                            "Toggle label for locking the Safe Box when app goes to background."
                                    )
                                )
                                .padding(.bottom, 0.5)
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.lock_when_app_hidden.description",
                                        defaultValue: "When switching apps or locking the screen",
                                        bundle: #bundle,
                                        comment:
                                            "Description under the lock-on-background toggle in Safe Box settings."
                                    )
                                )
                                .font(.system(.footnote))
                                .foregroundColor(Color(.secondaryLabel))
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
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.destroy_repo.label",
                                        defaultValue: "Destroy Safe Box…",
                                        bundle: #bundle,
                                        comment:
                                            "Row label in Safe Box settings that opens the destroy Safe Box dialog."
                                    )
                                )
                                .padding(.bottom, 0.5)
                                .foregroundColor(Color(.label))
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_info.destroy_repo.description",
                                        defaultValue: "Verify Safe Key and destroy the Safe box",
                                        bundle: #bundle,
                                        comment:
                                            "Description under the destroy Safe Box row in Safe Box settings."
                                    )
                                )
                                .font(.system(.footnote))
                                .foregroundColor(Color(.secondaryLabel))
                            }
                        }
                    }
                    .padding(.vertical, 2)
                }
            }
            .navigationTitle(repo.name)
        }
    }

    func repoAutoLockAfterDisplay(_ after: RepoAutoLockAfter) -> LocalizedStringResource {
        switch after {
        case .noLimit:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.no_time_limit",
                defaultValue: "No time limit",
                bundle: #bundle,
                comment:
                    "Auto-lock timeout option label meaning the safe box should not auto-lock by time."
            )
        case .inactive1Minute:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_1_minute",
                defaultValue: "1 minute of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for one minute of inactivity."
            )
        case .inactive5Mininutes:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_5_minutes",
                defaultValue: "5 minutes of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for five minutes of inactivity."
            )
        case .inactive10Minutes:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_10_minutes",
                defaultValue: "10 minutes of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for ten minutes of inactivity."
            )
        case .inactive30Minutes:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_30_minutes",
                defaultValue: "30 minutes of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for thirty minutes of inactivity."
            )
        case .inactive1Hour:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_1_hour",
                defaultValue: "1 hour of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for one hour of inactivity."
            )
        case .inactive2Hours:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_2_hours",
                defaultValue: "2 hours of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for two hours of inactivity."
            )
        case .inactive4Hours:
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.inactive_4_hours",
                defaultValue: "4 hours of inactivity",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for four hours of inactivity."
            )
        case .custom(let seconds):
            return LocalizedStringResource(
                "ios.repo_info.auto_lock_after.custom",
                defaultValue: "Custom (\(seconds) seconds)",
                bundle: #bundle,
                comment: "Auto-lock timeout option label for a custom number of inactive seconds."
            )
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
