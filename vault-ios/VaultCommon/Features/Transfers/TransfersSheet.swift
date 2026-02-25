import SwiftUI
import VaultMobile

public struct TransfersSheet: View {
    public let container: Container
    public var onDismiss: () -> Void

    public var body: some View {
        NavigationView {
            TransfersView(container: container)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            onDismiss()
                        } label: {
                            Text(
                                LocalizedStringResource(
                                    "ios.transfers.hide.button",
                                    defaultValue: "Hide",
                                    bundle: #bundle,
                                    comment: "Toolbar button that dismisses the transfers sheet."
                                )
                            )
                            .bold()
                        }
                    }
                }
        }
    }
}
