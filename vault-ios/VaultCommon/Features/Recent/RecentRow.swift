import SwiftUI

public struct RecentRow: View {
    public var body: some View {
        HStack {
            Image(systemName: "clock").renderingMode(.template)
                .foregroundColor(Color(.label)).padding(7)
            Text("Recent")
                .foregroundColor(Color(.label))
        }
    }
}

public struct RecentRow_Previews: PreviewProvider {
    static public var previews: some View {
        Group {
            RecentRow()
        }.previewLayout(.fixed(width: 300, height: 70))
    }
}
