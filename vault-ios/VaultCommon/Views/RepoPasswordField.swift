import SwiftUI

public struct RepoPasswordField: View {
    public enum Field {
        case secure
        case plain
    }

    @Binding var text: String
    public var onSubmit: (() -> Void)? = nil
    public var autoFocus = false
    public var inline = false
    public var label: LocalizedStringResource? = nil

    @State private var isPasswordVisible: Bool = false
    @FocusState private var focusedField: Field?

    private func togglePasswordVisible() {
        isPasswordVisible.toggle()

        focus()
    }

    private func focus() {
        if isPasswordVisible {
            focusedField = .plain
        } else {
            focusedField = .secure
        }
    }

    public var body: some View {
        HStack {
            ZStack {
                secureField
                textField
            }

            Button {
                togglePasswordVisible()
            } label: {
                Image(systemName: isPasswordVisible ? "eye" : "eye.slash").accentColor(.gray)
            }
            .if(!inline) { view in
                view.padding()
            }
        }
        .if(!inline) { view in
            view
                .frame(height: 21)
                .padding(EdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 0))
                .background(Color(.systemFill))
                .cornerRadius(5.0)
        }
        .onAppear {
            if autoFocus {
                focus()
            }
        }
    }

    @ViewBuilder var secureField: some View {
        SecureField(String(stringLiteral: ""), text: $text)
            .textInputAutocapitalization(.never)
            .keyboardType(.asciiCapable)
            .autocorrectionDisabled()
            .opacity(isPasswordVisible ? 0 : 1)
            .focused($focusedField, equals: .secure)
            .if(onSubmit != nil) { view in
                view.onSubmit(onSubmit!)
            }
            .onTapGesture {
                focusedField = .secure
            }
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.views.repo_password.a11y.label",
                        defaultValue: "Safe Key",
                        bundle: #bundle,
                        comment: "Accessibility label for the secure Safe Key input field."
                    )
                )
            )
            .overlay(alignment: .leading) {
                if text.isEmpty {
                    Text(
                        label
                            ?? LocalizedStringResource(
                                "ios.views.repo_password.placeholder",
                                defaultValue: "Safe Key",
                                bundle: #bundle,
                                comment: "Placeholder text inside the Safe Key input field."
                            )
                    )
                    .foregroundColor(Color(.placeholderText))
                    .opacity(isPasswordVisible ? 0 : 1)
                }
            }
    }

    @ViewBuilder var textField: some View {
        TextField(String(stringLiteral: ""), text: $text)
            .textInputAutocapitalization(.never)
            .keyboardType(.asciiCapable)
            .autocorrectionDisabled()
            .opacity(isPasswordVisible ? 1 : 0)
            .focused($focusedField, equals: .plain)
            .if(onSubmit != nil) { view in
                view.onSubmit(onSubmit!)
            }
            .onTapGesture {
                focusedField = .plain
            }
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.views.repo_password.a11y.label",
                        defaultValue: "Safe Key",
                        bundle: #bundle,
                        comment: "Accessibility label for the secure Safe Key input field."
                    )
                )
            )
            .overlay(alignment: .leading) {
                if text.isEmpty {
                    Text(
                        label
                            ?? LocalizedStringResource(
                                "ios.views.repo_password.placeholder",
                                defaultValue: "Safe Key",
                                bundle: #bundle,
                                comment: "Placeholder text inside the Safe Key input field."
                            )
                    )
                    .foregroundColor(Color(.placeholderText))
                    .opacity(isPasswordVisible ? 1 : 0)
                }
            }
    }
}

public struct RepoPasswordFieldPreview: View {
    @State var text: String = ""
    var inline: Bool

    public var body: some View {
        VStack {
            RepoPasswordField(text: $text, inline: inline)
        }.padding(10)
    }
}

public struct RepoPasswordField_Previews: PreviewProvider {
    static public var previews: some View {
        Group {
            RepoPasswordFieldPreview(inline: false)
                .previewDisplayName("Box")
            RepoPasswordFieldPreview(inline: true)
                .previewDisplayName("Inline")
        }
    }
}
