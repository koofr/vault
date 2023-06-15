import SwiftUI

public struct FormInfoSheet: View {
    public let title: String
    public let text: String
    public let onDismiss: () -> Void

    public var body: some View {
        NavigationView {
            VStack {
                HStack {
                    Text(text)
                    Spacer()
                }
                Spacer()
            }
            .padding()
            .navigationTitle(title)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Dismiss") {
                        onDismiss()
                    }
                }
            }
        }
    }
}

struct FormInfoSheet_Previews: PreviewProvider {
    static var previews: some View {
        FormInfoSheet(title: "Title", text: "Text", onDismiss: {})
    }
}
