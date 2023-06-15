import SwiftUI

extension View {
    public func navigationStackNavController<Route: Hashable>(_ navController: NavController<Route>)
        -> some View
    {
        navigationStackInfo(didShow: {
            navController.navigationControllerDidShow()
        })
        .task {
            do {
                try await Task.sleep(for: Duration.seconds(2))

                DispatchQueue.main.async {
                    // fallback if patching navigationStack fails
                    navController.navigationControllerDidShow()
                }
            } catch {}
        }
    }
}
