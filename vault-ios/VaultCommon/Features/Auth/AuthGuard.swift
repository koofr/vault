import SwiftUI
import VaultMobile

public enum AuthGuardState {
    case loading
    case navigation(navController: MainVaultNavController)
    case landing
}

public class AuthGuardViewModel: ObservableObject {
    public let container: Container

    @Published public var state: VaultMobile.Subscription<AuthGuardState>

    public init(container: Container) {
        self.container = container

        state = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.oauth2StatusSubscribe(cb: cb)
            },
            getData: { v, id in
                switch v.oauth2StatusData(id: id) {
                case .some(.loading(_)): return .loading
                case .some(.loaded):
                    let navController = MainVaultNavController(
                        navController: MainNavController(rootRoute: .repos),
                        mobileVault: container.mobileVault)

                    return .navigation(navController: navController)
                default:
                    // hide all sheets on logout
                    container.sheets.hideAll()

                    return .landing
                }
            })
    }
}

public struct AuthGuard: View {
    @ObservedObject var vm: AuthGuardViewModel

    @ObservedObject private var state: Subscription<AuthGuardState>

    public init(vm: AuthGuardViewModel) {
        self.vm = vm

        state = vm.state
    }

    public var body: some View {
        switch state.data! {
        case .loading:
            LoadingView()
        case .navigation(let navController):
            MainNavigation(container: vm.container, navController: navController.navController)
        case .landing:
            LandingScreen(container: vm.container)
        }
    }
}
