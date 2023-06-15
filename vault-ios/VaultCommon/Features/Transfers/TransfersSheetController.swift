import Foundation
import VaultMobile

public class TransfersSheetController {
    private weak var container: Container?
    private var transfersIsActive: Subscription<Bool>?
    private var callShowWhenActive = false

    public init() {}

    public func setContainer(container: Container) {
        self.container = container

        let transfersIsActive = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersIsActiveSubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersIsActiveData(id: id)
            })

        transfersIsActive.setOnData { [weak self] isActive in
            if let self = self {
                if let isActive = isActive {
                    if isActive && self.callShowWhenActive {
                        self.callShowWhenActive = false

                        self.show()
                    }

                    if !isActive {
                        self.hide()
                    }
                }
            }
        }

        self.transfersIsActive = transfersIsActive
    }

    public func show() {
        if let container = container {
            container.sheets.show(name: "transfers") { _, hide in
                TransfersSheet(container: container, onDismiss: hide)
            }
        }
    }

    public func showWhenActive() {
        if let container = container {
            if container.sheets.isVisible(name: "transfers") {
                return
            }
        }

        if let transfersIsActive = transfersIsActive {
            if transfersIsActive.data == true {
                show()
            } else {
                callShowWhenActive = true
            }
        }
    }

    public func hide() {
        if let container = container {
            container.sheets.hide(name: "transfers")
        }
    }
}
