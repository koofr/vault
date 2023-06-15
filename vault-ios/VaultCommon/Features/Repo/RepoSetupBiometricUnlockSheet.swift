import SwiftUI
import VaultMobile

public class RepoSetupBiometricUnlockSheetViewModel: ObservableObject {
    public let container: Container
    public let repoId: String

    public let unlockId: UInt32

    public init(container: Container, repoId: String) {
        self.container = container
        self.repoId = repoId

        self.unlockId = container.mobileVault.repoUnlockCreate(
            repoId: repoId, options: RepoUnlockOptions(mode: .verify))
    }

    deinit {
        container.mobileVault.repoUnlockDestroy(unlockId: unlockId)
    }

    public func enableBiometricUnlock(password: String, onEnabled: () -> Void) {
        do {
            try container.keychainRepoPasswordStorage.setPassword(
                repoId: repoId, password: password)

            onEnabled()
        } catch {
            print(
                "RepoSetupBiometricUnlock keychainRepoPasswordStorage.setPassword error: \(error)")
        }
    }
}

public struct RepoSetupBiometricUnlockSheet: View {
    public var vm: RepoSetupBiometricUnlockSheetViewModel
    public var onDismiss: () -> Void

    @ObservedObject private var info: Subscription<RepoUnlockInfo>

    public init(vm: RepoSetupBiometricUnlockSheetViewModel, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.onDismiss = onDismiss

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
        NavigationView {
            VStack {
                if let info = info.data {
                    RepoUnlockForm(
                        info: info,
                        onUnlock: { password in
                            vm.container.mobileVault.repoUnlockUnlock(
                                unlockId: vm.unlockId, password: password,
                                cb: RepoUnlockUnlockedFn({
                                    vm.enableBiometricUnlock(
                                        password: password, onEnabled: onDismiss)
                                }))
                        },
                        message: "Enter your Safe Key to setup biometric unlock")
                }
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        onDismiss()
                    }
                }
            }
        }
    }
}
