import SwiftUI
import VaultMobile

struct RepoCreateRow: View {
    var body: some View {
        HStack {
            Image(systemName: "plus.circle").padding(6).tint(Color(.label))
            Text(
                LocalizedStringResource(
                    "ios.repos.create_new.label",
                    defaultValue: "Create new",
                    bundle: #bundle,
                    comment: "Label for the create-new Safe Box row in the Safe Boxes list."
                )
            )
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
