import SwiftUI

// List has a bug where it displays incorrect items if the items change while
// the list was hidden
public struct FrozenList<Data, ID, RowContent>: View
where Data: RandomAccessCollection, ID: Hashable, RowContent: View {
    var data: Data
    var id: KeyPath<Data.Element, ID>
    var rowContent: (Data.Element) -> RowContent

    @State var frozenData: Data?

    public init(
        _ data: Data, id: KeyPath<Data.Element, ID>,
        @ViewBuilder rowContent: @escaping (Data.Element) -> RowContent
    ) {
        self.data = data
        self.id = id
        self.rowContent = rowContent
    }

    public var body: some View {
        List(frozenData ?? data, id: id, rowContent: rowContent)
            .onAppear {
                frozenData = nil
            }
            .onDisappear {
                frozenData = data
            }
    }
}
