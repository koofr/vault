import SwiftUI
import VaultMobile

public struct TransfersSheet: View {
    public let container: Container
    public var onDismiss: () -> Void

    public var body: some View {
        NavigationView {
            TransfersView(container: container)
                .navigationBarTitle("", displayMode: .inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            onDismiss()
                        } label: {
                            Text("Hide").bold()
                        }
                    }
                }
        }
    }
}
