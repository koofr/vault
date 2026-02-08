import SwiftUI
import VaultMobile

public struct ShareExtContentView: View {
    @ObservedObject var vm: ShareExtScreenViewModel

    @ObservedObject private var intlCurrentLocale: Subscription<IntlLocale>

    public init(vm: ShareExtScreenViewModel) {
        self.vm = vm

        self.intlCurrentLocale = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.intlCurrentLocaleSubscribe(cb: cb)
            },
            getData: { v, id in
                v.intlCurrentLocaleData(id: id)
            })
    }

    public var body: some View {
        if let currentLocale = intlCurrentLocale.data {
            ZStack {
                ShareExtScreen(vm: vm)

                Overlays(container: vm.container)
            }
            .environment(\.locale, Locale(identifier: currentLocale.locale))
        }
    }
}
