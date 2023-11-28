// from https://stackoverflow.com/a/76329393/141214

import SwiftUI
import UIKit

struct WindowReader: UIViewRepresentable {
    let handler: (UIWindow?) -> Void

    @MainActor
    final class View: UIView {
        var didMoveToWindowHandler: ((UIWindow?) -> Void)

        init(didMoveToWindowHandler: (@escaping (UIWindow?) -> Void)) {
            self.didMoveToWindowHandler = didMoveToWindowHandler
            super.init(frame: .null)
            backgroundColor = .clear
            isUserInteractionEnabled = false
        }

        @available(*, unavailable)
        required init?(coder: NSCoder) {
            fatalError("init(coder:) has not been implemented")
        }

        override func didMoveToWindow() {
            super.didMoveToWindow()
            didMoveToWindowHandler(window)
        }
    }

    func makeUIView(context: Context) -> View {
        .init(didMoveToWindowHandler: handler)
    }

    func updateUIView(_ uiView: View, context: Context) {
        uiView.didMoveToWindowHandler = handler
    }
}
