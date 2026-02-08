import SwiftUI
import VaultMobile

public struct ShareExtContentView: View {
    @ObservedObject var vm: ShareExtScreenViewModel

    public init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    public var body: some View {
        ZStack {
            ShareExtScreen(vm: vm)

            Overlays(container: vm.container)
        }
    }
}
