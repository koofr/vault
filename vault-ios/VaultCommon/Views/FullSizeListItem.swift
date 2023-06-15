import SwiftUI

public struct FullSizeListItem<Content>: View
where Content: View {
    let content: () -> Content

    public init(@ViewBuilder content: @escaping () -> Content) {
        self.content = content
    }

    public var body: some View {
        GeometryReader { listGeometry in
            List {
                content()
                    .frame(height: listGeometry.size.height)
                    .listRowSeparator(.hidden)
                    .listRowInsets(.init(top: 0, leading: 0, bottom: 0, trailing: 0))
            }
        }
    }
}
