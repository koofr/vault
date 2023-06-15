import SwiftUI
import VaultMobile

public struct DialogView: View {
    public let container: Container
    public let dialogId: UInt32

    @ObservedObject private var dialog: Subscription<Dialog>
    @State private var localInputValue: String
    @State private var inputValueSelected: String?
    @State private var inputValueSelectedChanged: Bool
    @State private var inputTextField: UITextField?

    public init(container: Container, dialogId: UInt32) {
        self.container = container
        self.dialogId = dialogId

        let dialog = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.dialogsDialogSubscribe(dialogId: dialogId, cb: cb)
            },
            getData: { v, id in
                v.dialogsDialogData(id: id)
            })
        self.dialog = dialog

        self.localInputValue = dialog.data?.inputValue ?? ""
        self.inputValueSelected = dialog.data?.inputValueSelected
        self.inputValueSelectedChanged = false
    }

    private func handleInputValueSelection(_ textField: UITextField) {
        if !inputValueSelectedChanged {
            if let selected = self.inputValueSelected {
                if let to = textField.position(
                    from: textField.beginningOfDocument, offset: selected.count)
                {
                    // we just move the cursor, not select text (from is not set
                    // to textField.beginningOfDocument) because if the selected
                    // text is too long it breaks the UI (does not show
                    // selection, just breaks the input until you click outside
                    // of invisible selection)
                    textField.selectedTextRange = textField.textRange(from: to, to: to)
                }
            }

            inputValueSelectedChanged = true
        }
    }

    private func getConfirmButtonRole(_ dialog: Dialog) -> ButtonRole? {
        switch dialog.confirmButtonStyle {
        case .primary:
            return nil
        case .destructive:
            return .destructive
        }
    }

    private func updateInputTextField(_ dialog: Dialog) {
        if let inputTextField = inputTextField {
            inputTextField.textColor = dialog.confirmButtonEnabled ? .label : .systemRed
        }
    }

    public var body: some View {
        if let dialog = dialog.data {
            let isPresented = Binding(
                get: { true },
                set: { value in
                    container.mobileVault.dialogsCancel(dialogId: dialogId)
                })

            Color.clear
                .alert(dialog.title, isPresented: isPresented) {
                    buildAlert()
                } message: {
                    if let message = dialog.message {
                        Text(message)
                    }
                }
        }
    }

    @ViewBuilder func buildAlert() -> some View {
        let localInputValue = Binding(
            get: { self.localInputValue },
            set: { value in
                self.localInputValue = value

                container.mobileVault.dialogsSetInputValue(dialogId: dialogId, value: value)
            })

        if let dialog = dialog.data {
            let _ = updateInputTextField(dialog)

            switch dialog.typ {
            case .prompt:
                TextField("", text: localInputValue)
                    .onReceive(
                        NotificationCenter.default.publisher(
                            for: UITextField.textDidBeginEditingNotification)
                    ) { obj in
                        if let textField = obj.object as? UITextField {
                            handleInputValueSelection(textField)

                            inputTextField = textField

                            updateInputTextField(dialog)
                        }
                    }

            default: EmptyView()
            }

            if let cancelButtonText = dialog.cancelButtonText {
                Button(role: .cancel) {
                    container.mobileVault.dialogsCancel(dialogId: dialogId)
                } label: {
                    Text(cancelButtonText)
                }
                .keyboardShortcut(.cancelAction)
            }

            Button(role: getConfirmButtonRole(dialog)) {
                container.mobileVault.dialogsConfirm(dialogId: dialogId)
            } label: {
                // we cannot conditionally style or change the button text
                Text(dialog.confirmButtonText)
            }
            .keyboardShortcut(.defaultAction)
        }
    }
}
