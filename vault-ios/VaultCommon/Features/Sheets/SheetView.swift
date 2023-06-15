import SwiftUI

public struct SheetView<ExtraContent>: View where ExtraContent: View {
    @ObservedObject public var sheet: Sheet
    @ViewBuilder private var extraContent: () -> ExtraContent

    public init(sheet: Sheet, @ViewBuilder extraContent: @escaping () -> ExtraContent) {
        self.sheet = sheet
        self.extraContent = extraContent
    }

    public var body: some View {
        let isPresented = Binding(
            get: {
                sheet.isPresented
            },
            set: { value in
                if !value {
                    sheet.hide()
                }
            })

        Color.clear
            .sheet(isPresented: isPresented, onDismiss: sheet.onDismiss) {
                ZStack {
                    sheet.getContent()

                    extraContent()
                }
            }
            .onAppear {
                sheet.onAppear()
            }
    }
}
