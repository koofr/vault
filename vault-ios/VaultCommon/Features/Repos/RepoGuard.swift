import SwiftUI
import VaultMobile

public enum RepoGuardState {
    case loading
    case locked(vm: RepoUnlockScreenViewModel)
    case unlocked
    case error(error: String)

    public var repoState: RepoState? {
        switch self {
        case .loading: return nil
        case .locked(_): return RepoState.locked
        case .unlocked: return RepoState.unlocked
        case .error(_): return nil
        }
    }
}

public class RepoGuardViewModel: ObservableObject {
    public let container: Container
    public let repoId: String
    public let setupBiometricUnlockVisible: Bool

    private let repoInfo: Subscription<RepoInfo>

    @Published public var state: RepoGuardState = .loading

    public init(
        container: Container,
        repoId: String, setupBiometricUnlockVisible: Bool
    ) {
        self.container = container
        self.repoId = repoId
        self.setupBiometricUnlockVisible = setupBiometricUnlockVisible

        self.repoInfo = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.reposRepoSubscribe(repoId: repoId, cb: cb)
            },
            getData: { v, id in
                v.reposRepoData(id: id)
            })

        self.repoInfo.setOnData { data in
            if let info = data {
                self.updateState(info: info)
            }
        }
    }

    public func updateState(info: RepoInfo) {
        switch info.status {
        case .initial, .loading(loaded: false):
            state = .loading
        case .loading(loaded: true), .loaded:
            if let repo = info.repo {
                if state.repoState != repo.state {
                    switch repo.state {
                    case .locked:
                        state = .locked(
                            vm: RepoUnlockScreenViewModel(
                                container: container, repoId: repoId,
                                setupBiometricUnlockVisible: setupBiometricUnlockVisible))
                    case .unlocked:
                        state = .unlocked
                    }
                }
            } else {
                state = .loading
            }
        case .err(let error, loaded: _):
            state = .error(error: error)
        }
    }
}

public struct RepoGuard<Content>: View
where Content: View {
    var content: () -> Content

    @ObservedObject var vm: RepoGuardViewModel

    public init(vm: RepoGuardViewModel, @ViewBuilder content: @escaping () -> Content) {
        self.vm = vm
        self.content = content
    }

    public var body: some View {
        switch vm.state {
        case .loading: LoadingView()
        case .locked(let vm):
            RepoUnlockScreen(vm: vm, onUnlock: {})
        case .unlocked:
            content()
        case .error(let error):
            ErrorView(
                errorText: error,
                onRetry: {
                    vm.container.mobileVault.load()
                })
        }
    }
}
