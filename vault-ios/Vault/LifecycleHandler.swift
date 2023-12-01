import UIKit
import VaultCommon

class LifecycleHandler: NSObject {
    let container: Container

    init(container: Container) {
        self.container = container

        super.init()

        NotificationCenter.default.addObserver(
            self, selector: #selector(willEnterForeground),
            name: UIApplication.willEnterForegroundNotification, object: nil)
        NotificationCenter.default.addObserver(
            self, selector: #selector(didEnterBackground),
            name: UIApplication.didEnterBackgroundNotification, object: nil)
    }

    deinit {
        NotificationCenter.default.removeObserver(
            self, name: UIApplication.willEnterForegroundNotification, object: nil)
        NotificationCenter.default.removeObserver(
            self, name: UIApplication.didEnterBackgroundNotification, object: nil)
    }

    @objc func willEnterForeground() {
        container.mobileVault.appVisible()
    }

    @objc func didEnterBackground() {
        container.mobileVault.appHidden()
    }
}
