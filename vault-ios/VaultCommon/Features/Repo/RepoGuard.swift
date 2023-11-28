import SwiftUI
import VaultMobile

public enum RepoGuardState {
    case loading
    case locked(vm: RepoUnlockScreenViewModel)
    case unlocked
    case error(error: String)

    public func matches(isLocked: Bool) -> Bool {
        switch self {
        case .loading: return false
        case .locked(_): return isLocked == true
        case .unlocked: return isLocked == false
        case .error(_): return false
        }
    }
}

public protocol WithRepoGuardViewModel {
    var repoGuardViewModel: RepoGuardViewModel { get }
}

public class RepoGuardViewModel: ObservableObject {
    public let container: Container
    public let repoId: String
    public let setupBiometricUnlockVisible: Bool

    @Published public var state: RepoGuardState = .loading

    public init(
        container: Container,
        repoId: String, setupBiometricUnlockVisible: Bool
    ) {
        self.container = container
        self.repoId = repoId
        self.setupBiometricUnlockVisible = setupBiometricUnlockVisible
    }

    public func update(repoStatus: Status, isLocked: Bool) {
        switch repoStatus {
        case .initial, .loading(loaded: false):
            state = .loading
        case .loading(loaded: true), .loaded:
            if !state.matches(isLocked: isLocked) {
                if isLocked {
                    state = .locked(
                        vm: RepoUnlockScreenViewModel(
                            container: container, repoId: repoId,
                            setupBiometricUnlockVisible: setupBiometricUnlockVisible))
                } else {
                    state = .unlocked
                }
            }
        case .err(let error, loaded: _):
            state = .error(error: error)
        }
    }
}

public struct RepoGuard<Content, ViewModel>: View
where Content: View, ViewModel: WithRepoGuardViewModel {
    let vm: ViewModel
    var content: (ViewModel) -> Content

    @ObservedObject var repoGuardViewModel: RepoGuardViewModel

    public init(_ vm: ViewModel, @ViewBuilder content: @escaping (ViewModel) -> Content) {
        self.vm = vm
        self.content = content

        self.repoGuardViewModel = vm.repoGuardViewModel
    }

    public var body: some View {
        switch repoGuardViewModel.state {
        case .loading: LoadingView()
        case .locked(let vm):
            RepoUnlockScreen(vm: vm, onUnlock: {})
        case .unlocked:
            UnlockedRepoWrapper(
                container: repoGuardViewModel.container, repoId: repoGuardViewModel.repoId
            ) {
                content(vm)
            }
        case .error(let error):
            ErrorView(
                errorText: error,
                onRetry: {
                    repoGuardViewModel.container.mobileVault.load()
                })
        }
    }
}
