import SwiftUI

public struct FormInfoSheet: View {
    public let title: LocalizedStringResource
    public let text: LocalizedStringResource
    public let onDismiss: () -> Void

    public var body: some View {
        NavigationView {
            ScrollView {
                VStack {
                    HStack {
                        Text(text)
                        Spacer()
                    }
                    Spacer()
                }
            }
            .padding()
            .navigationTitle(Text(title))
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        onDismiss()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.views.form_info.dismiss.button",
                                defaultValue: "Dismiss",
                                bundle: #bundle,
                                comment: "Toolbar button that closes a form info/help sheet."
                            )
                        )
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
