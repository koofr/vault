import Foundation
import SwiftUINavController
import VaultMobile

/// A wrapper around `SwiftUINavController.NavController` that manages
/// navigation state in the presence of active dialogs.
///
/// This class monitors the visibility of dialogs and updates the navigation
/// controller via `alertDisplayed` and `alertHidden`. This workaround addresses
/// a known issue in SwiftUI's `NavigationStack` where navigation changes are
/// ignored while alerts are visible.
public class VaultNavController<Route: Equatable & Hashable>: ObservableObject {
    @Published var navController: NavController<Route>
    let mobileVault: MobileVault

    private let dialogs: Subscription<[UInt32]>

    private var currentDialogs: Set<UInt32>

    public init(navController: NavController<Route>, mobileVault: MobileVault) {
        self.navController = navController
        self.mobileVault = mobileVault

        dialogs = Subscription(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.dialogsSubscribe(cb: cb)
            },
            getData: { v, id in
                v.dialogsData(id: id)
            })

        currentDialogs = Set()

        dialogs.setOnData { [weak self] data in
            if let self = self {
                let newDialogs: Set<UInt32> =
                    if let data = data {
                        Set(data)
                    } else {
                        Set()
                    }

                // Find dialogs that are new (in data but not in currentDialogs)
                let addedDialogs = newDialogs.subtracting(self.currentDialogs)
                for dialogId in addedDialogs {
                    self.navController.alertDisplayed("\(dialogId)")
                }

                // Find dialogs that are removed (in currentDialogs but not in data)
                let removedDialogs = self.currentDialogs.subtracting(newDialogs)
                for dialogId in removedDialogs {
                    self.navController.alertHidden("\(dialogId)")
                }

                // Update current state
                self.currentDialogs = newDialogs
            }
        }
    }
}
