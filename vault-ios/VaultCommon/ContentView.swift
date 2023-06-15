import SwiftUI
import VaultMobile

public struct ContentView: View {
    public let container: Container

    public init(container: Container) {
        self.container = container
    }

    public var body: some View {
        ZStack {
            AuthGuard(container: container)

            Overlays(container: container)
        }
    }
}
