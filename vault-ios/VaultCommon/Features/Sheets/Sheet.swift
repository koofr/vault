import SwiftUI

public class Sheet: ObservableObject {
    public let id: Int
    public let name: String?
    public let content: (Sheet) -> AnyView
    public let onHide: (() -> Void)?
    public let onDismiss: () -> Void

    @Published public var isPresented: Bool
    private var hasAppeared: Bool

    public init(
        id: Int, name: String?, content: @escaping (Sheet) -> AnyView, onHide: (() -> Void)?,
        onDismiss: @escaping () -> Void
    ) {
        self.id = id
        self.name = name
        self.content = content
        self.onHide = onHide
        self.onDismiss = onDismiss

        self.isPresented = false
        self.hasAppeared = false
    }

    public func getContent() -> AnyView {
        return content(self)
    }

    public func hide() {
        isPresented = false

        if let onHide = onHide {
            onHide()
        }
    }

    public func onAppear() {
        if !hasAppeared {
            hasAppeared = true
            isPresented = true
        }
    }
}
