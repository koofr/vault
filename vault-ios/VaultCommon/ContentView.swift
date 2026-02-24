import SwiftUI
import VaultMobile

public class ContentViewModel: ObservableObject {
    public let container: Container
    public let authGuardVm: AuthGuardViewModel

    public init(container: Container) {
        self.container = container

        authGuardVm = AuthGuardViewModel(container: container)
    }
}

public struct ContentView: View {
    public let vm: ContentViewModel

    public init(vm: ContentViewModel) {
        self.vm = vm
    }

    public var body: some View {
        ZStack {
            AuthGuard(vm: vm.authGuardVm)

            Overlays(container: vm.container)
        }
    }
}
