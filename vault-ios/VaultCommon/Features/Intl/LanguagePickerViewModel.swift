import Foundation
import VaultMobile

public class LanguagePickerViewModel: ObservableObject {
    public let container: Container

    @Published public var locales: Subscription<[IntlLocale]>
    @Published public var currentLocale: Subscription<IntlLocale>

    public init(container: Container) {
        self.container = container

        self.locales = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.intlLocalesSubscribe(cb: cb)
            },
            getData: { v, id in
                v.intlLocalesData(id: id)
            })

        self.currentLocale = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.intlCurrentLocaleSubscribe(cb: cb)
            },
            getData: { v, id in
                v.intlCurrentLocaleData(id: id)
            })
    }

    public func changeLocale(_ locale: IntlLocale) {
        container.mobileVault.intlChangeLocale(strategy: .exact(locale: locale.locale))
    }
}
