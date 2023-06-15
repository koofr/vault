import SwiftUI
import VaultMobile

public struct RepoRow: View {
    public var repo: Repo

    public var body: some View {
        HStack {
            Image(repo.state == RepoState.locked ? "locked" : "unlocked").renderingMode(.template)
                .foregroundColor(Color(.label)).padding(7)
            Text(repo.name)
                .truncationMode(.middle)
                .foregroundColor(Color(.label))
            Spacer()
        }
    }
}

public struct RepoRow_Previews: PreviewProvider {
    static public var previews: some View {
        Group {
            RepoRow(repo: PreviewsData.repos[0])
            RepoRow(repo: PreviewsData.repos[1])
        }.previewLayout(.fixed(width: 300, height: 70))
    }
}
