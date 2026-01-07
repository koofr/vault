import SwiftUI

struct ReadOnlyTextEditor: UIViewRepresentable {
    let text: String

    func makeUIView(context: Context) -> UITextView {
        let textView = UITextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.isScrollEnabled = true
        textView.backgroundColor = .clear
        textView.font = ReadOnlyTextEditor.monospacedFont()
        textView.adjustsFontForContentSizeCategory = true
        return textView
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        uiView.text = text
    }

    static func monospacedFont() -> UIFont {
        let base = UIFont.preferredFont(forTextStyle: .callout)
        let descriptor = base.fontDescriptor.withDesign(.monospaced)!
        return UIFont(descriptor: descriptor, size: 0)
    }
}
