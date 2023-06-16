import SwiftUI
import VaultMobile

public class RepoUnlockScreenViewModel: ObservableObject {
    public let container: Container
    public let repoId: String
    public let setupBiometricUnlockVisible: Bool

    public let unlockId: UInt32

    @Published var canSetupBiometricUnlock: Bool

    public init(
        container: Container, repoId: String, setupBiometricUnlockVisible: Bool,
        onUnlock: (() -> Void)? = nil
    ) {
        self.container = container
        self.repoId = repoId
        self.setupBiometricUnlockVisible = setupBiometricUnlockVisible

        self.unlockId = container.mobileVault.repoUnlockCreate(
            repoId: repoId, options: RepoUnlockOptions(mode: .unlock))

        self.canSetupBiometricUnlock = false
    }

    deinit {
        container.mobileVault.repoUnlockDestroy(unlockId: unlockId)
    }

    public func biometricUnlock(onUnlock: @escaping () -> Void) {
        do {
            if let password = try container.keychainRepoPasswordStorage.getPassword(repoId: repoId)
            {
                container.mobileVault.repoUnlockUnlock(
                    unlockId: unlockId, password: password,
                    cb: RepoUnlockUnlockedFn(onUnlock))
            } else {
                DispatchQueue.main.async {
                    self.canSetupBiometricUnlock = true
                }
            }
        } catch KeychainRepoPasswordStorageError.canceled {

        } catch {
            print("RepoUnlock keychainRepoPasswordStorage.getPassword error: \(error)")
        }
    }

    public func setupBiometricUnlock(onUnlock: @escaping () -> Void) {
        container.sheets.show(
            viewModel: RepoSetupBiometricUnlockSheetViewModel(
                container: container, repoId: repoId),
            onHide: {
                self.biometricUnlock(onUnlock: onUnlock)
            },
            content: { setupVm, hide in
                RepoSetupBiometricUnlockSheet(vm: setupVm, onDismiss: hide)
            })
    }
}

public struct RepoUnlockScreen: View {
    @ObservedObject var vm: RepoUnlockScreenViewModel
    public let onUnlock: () -> Void

    @ObservedObject private var info: Subscription<RepoUnlockInfo>

    public init(vm: RepoUnlockScreenViewModel, onUnlock: @escaping () -> Void) {
        self.vm = vm
        self.onUnlock = onUnlock

        self.info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoUnlockInfoSubscribe(unlockId: vm.unlockId, cb: cb)
            },
            getData: { v, id in
                v.repoUnlockInfoData(id: id)
            })
    }

    public var body: some View {
        VStack {
            if let info = info.data {
                RepoUnlockForm(
                    info: info,
                    onUnlock: { password in
                        vm.container.mobileVault.repoUnlockUnlock(
                            unlockId: vm.unlockId, password: password,
                            cb: RepoUnlockUnlockedFn(onUnlock))
                    })

                if vm.canSetupBiometricUnlock && vm.setupBiometricUnlockVisible {
                    Button {
                        vm.setupBiometricUnlock(onUnlock: onUnlock)
                    } label: {
                        Text("Setup biometric unlock").padding(.top, 15).padding(.bottom, 15)
                    }
                }
            }
        }
        .navigationTitle("")
        .navigationBarTitleDisplayMode(.inline)
        .task {
            let vm = self.vm

            Task.detached {
                vm.biometricUnlock(onUnlock: onUnlock)
            }
        }
    }
}