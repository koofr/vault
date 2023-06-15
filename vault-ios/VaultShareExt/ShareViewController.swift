import SwiftUI
import UIKit
import VaultCommon

@objc(ShareViewController)
final class ShareViewController: UIViewController {
    private var container: Container? = nil
    private var appearCount: Int = 0

    override func viewDidLoad() {
        super.viewDidLoad()

        let itemProviders = getItemProviders()

        let container = Container(
            baseURL: ProcessInfo.processInfo.environment["VAULT_BASE_URL"],
            oauth2AuthBaseURL: ProcessInfo.processInfo.environment["VAULT_OAUTH2_AUTH_BASE_URL"],
            secureStorageJson: ProcessInfo.processInfo.environment["VAULT_SECURE_STORAGE"]
        )
        self.container = container

        let vm = ShareExtScreenViewModel(container: container) {
            self.extensionContext?.completeRequest(returningItems: nil, completionHandler: nil)
        }

        vm.handleItemProviders(itemProviders)

        let shareExtScreen = ShareExtScreen(vm: vm)

        let hostingController = UIHostingController(rootView: shareExtScreen)

        addChild(hostingController)

        view.addSubview(hostingController.view)

        hostingController.view.translatesAutoresizingMaskIntoConstraints = false

        NSLayoutConstraint.activate([
            hostingController.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            hostingController.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            hostingController.view.topAnchor.constraint(equalTo: view.topAnchor),
            hostingController.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        hostingController.didMove(toParent: self)

        NotificationCenter.default.addObserver(
            self, selector: #selector(willEnterForeground),
            name: NSNotification.Name.NSExtensionHostWillEnterForeground, object: nil)
    }

    deinit {
        NotificationCenter.default.removeObserver(
            self, name: NSNotification.Name.NSExtensionHostWillEnterForeground, object: nil)
    }

    @objc func willEnterForeground() {
        // if the user is not authenticated, then opens the app, logs in and
        // comes back here we should call load again
        container?.mobileVault.load()
    }

    private func getItemProviders() -> [NSItemProvider] {
        guard let items = self.extensionContext?.inputItems as? [NSExtensionItem] else {
            return []
        }

        return items.compactMap({ $0.attachments }).flatMap { $0 }
    }
}
