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
    public var label: String? = nil

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
        SecureField(label ?? "Safe Key", text: $text)
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
            .accessibilityLabel("Safe Key")
    }

    @ViewBuilder var textField: some View {
        TextField(label ?? "Safe Key", text: $text)
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
            .accessibilityLabel("Safe Key")
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
