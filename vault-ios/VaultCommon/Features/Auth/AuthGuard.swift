import SwiftUI
import VaultMobile

enum AuthGuardState {
    case loading
    case navigation(navController: MainNavController)
    case landing
}

public struct AuthGuard: View {
    public let container: Container

    @ObservedObject private var state: Subscription<AuthGuardState>

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
                    let navController = MainNavController(rootRoute: .repos)

                    return .navigation(navController: navController)
                default:
                    // hide all sheets on logout
                    container.sheets.hideAll()

                    return .landing
                }
            })
    }

    public var body: some View {
        switch state.data! {
        case .loading:
            LoadingView()
        case .navigation(let navController):
            MainNavigation(container: container, navController: navController)
        case .landing:
            LandingScreen(container: container)
        }
    }
}
