import SwiftUI

struct Overlays: View {
    private var container: Container
    @ObservedObject var sheets: Sheets

    public init(container: Container) {
        self.container = container
        self.sheets = container.sheets
    }

    var body: some View {
        ZStack {
            if !sheets.active {
                NotificationsView(container: container)

                DialogsView(container: container)
            }

            SheetsView(sheets: sheets.sheets.dropFirst(0)) {
                NotificationsView(container: container)

                DialogsView(container: container)
            }
        }
    }
}
