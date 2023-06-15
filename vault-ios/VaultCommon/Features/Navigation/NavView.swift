import SwiftUI

public struct NavView<Content, Route>: View where Content: View, Route: Hashable {
    public let navController: NavController<Route>
    public let routeContainer: RouteContainer<Route>
    @ViewBuilder var content: () -> Content

    public var body: some View {
        content()
            .onAppear {
                navController.onAppear(routeContainer)
            }
            .onDisappear {
                navController.onDisappear(routeContainer)
            }
    }
}
