import SwiftUI

public class Sheets: ObservableObject {
    @Published public var sheets: [Sheet]
    private var nextId: Int

    public var active: Bool {
        !sheets.isEmpty
    }

    public init() {
        self.sheets = []
        self.nextId = 1
    }

    public func show<Content, ViewModel>(
        name: String? = nil, viewModel: ViewModel = (), onHide: (() -> Void)? = nil,
        onDismiss: (() -> Void)? = nil,
        @ViewBuilder content: @escaping (ViewModel, @escaping () -> Void) -> Content
    ) where Content: View {
        if let name = name {
            if isVisible(name: name) {
                return
            }
        }

        let id = getNextId()

        let originalContent = content
        let content: (Sheet) -> AnyView = { sheet in
            AnyView(
                originalContent(
                    viewModel,
                    {
                        sheet.hide()
                    }))
        }

        let originalOnDismiss = onDismiss
        let onDismiss = { [weak self] in
            if let self = self {
                self.sheets.removeAll(where: { $0.id == id })
            }

            if let onDismiss = originalOnDismiss {
                onDismiss()
            }
        }

        let sheet = Sheet(
            id: id, name: name, content: content, onHide: onHide, onDismiss: onDismiss)

        self.sheets.append(sheet)
    }

    public func isVisible(name: String) -> Bool {
        return self.sheets.first(where: { $0.name == name }) != nil
    }

    public func hide(name: String) {
        if let sheet = self.sheets.first(where: { $0.name == name }) {
            sheet.hide()
        }
    }

    public func hideAll() {
        let sheets = self.sheets

        for sheet in sheets {
            sheet.hide()
        }
    }

    private func getNextId() -> Int {
        let id = nextId

        nextId += 1

        return id
    }
}
