import SwiftUI
import VaultMobile

struct RepoCreateRow: View {
    var body: some View {
        HStack {
            Image(systemName: "plus.circle").padding(6).tint(Color(.label))
            Text("Create new")
                .foregroundColor(Color(.label))
            Spacer()
        }
    }
}

struct RepoCreateRow_Previews: PreviewProvider {
    static var previews: some View {
        RepoCreateRow()
    }
}
