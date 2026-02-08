import SwiftUI
import VaultMobile

public class ContentViewModel: ObservableObject {
    public let container: Container
    public let authGuardVm: AuthGuardViewModel

    @Published public var intlCurrentLocale: Subscription<IntlLocale>

    public init(container: Container) {
        self.container = container

        authGuardVm = AuthGuardViewModel(container: container)

        intlCurrentLocale = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.intlCurrentLocaleSubscribe(cb: cb)
            },
            getData: { v, id in
                v.intlCurrentLocaleData(id: id)
            })
    }
}

public struct ContentView: View {
    public let vm: ContentViewModel

    @ObservedObject private var intlCurrentLocale: Subscription<IntlLocale>

    public init(vm: ContentViewModel) {
        self.vm = vm

        intlCurrentLocale = vm.intlCurrentLocale
    }

    public var body: some View {
        if let currentLocale = intlCurrentLocale.data {
            ZStack {
                AuthGuard(vm: vm.authGuardVm)

                Overlays(container: vm.container)
            }
            .environment(\.locale, Locale(identifier: currentLocale.locale))
        }
    }
}
