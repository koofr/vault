import SwiftUI
import VaultMobile

public struct TransfersButton: View {
    public let container: Container

    @ObservedObject private var transfersIsActive: Subscription<Bool>

    public init(container: Container) {
        self.container = container

        self.transfersIsActive = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersIsActiveSubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersIsActiveData(id: id)
            })
    }

    public var body: some View {
        if transfersIsActive.data == true {
            Button {
                container.transfersSheetController.show()
            } label: {
                Image(systemName: "arrow.up.arrow.down.square")
            }
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.transfers.show_transfers.a11y.label",
                        defaultValue: "Show transfers",
                        bundle: #bundle,
                        comment:
                            "Accessibility label for the toolbar button that opens the transfers sheet."
                    )
                )
            )
        }
    }
}
