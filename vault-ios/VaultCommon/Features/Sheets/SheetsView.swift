import SwiftUI

public struct SheetsView<ActiveContent>: View where ActiveContent: View {
    public var sheets: ArraySlice<Sheet>
    @ViewBuilder public var activeContent: () -> ActiveContent

    public init(
        sheets: ArraySlice<Sheet>, @ViewBuilder activeContent: @escaping () -> ActiveContent
    ) {
        self.sheets = sheets
        self.activeContent = activeContent
    }

    public var body: some View {
        if let sheet = sheets.first {
            let tail = sheets.dropFirst()

            SheetView(sheet: sheet) {
                if tail.isEmpty {
                    activeContent()
                } else {
                    SheetsView(sheets: tail, activeContent: activeContent)
                }
            }
        }
    }
}
