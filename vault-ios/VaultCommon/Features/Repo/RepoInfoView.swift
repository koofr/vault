import SwiftUI
import VaultMobile

public struct RepoInfoView: View {
    public var repo: Repo
    public var unlockRepo: () -> Void

    public var body: some View {
        VStack {
            Text(repo.name).font(.title)
            Text(repo.state == .locked ? "Locked" : "Unlocked").font(.title2)
            Spacer()
        }.toolbar {
            ToolbarItem(placement: .bottomBar) {
                Button("Unlock", action: unlockRepo)
            }
        }
    }
}

public struct RepoInfoView_Previews: PreviewProvider {
    static var previews: some View {
        RepoInfoView(repo: PreviewsData.repos[0], unlockRepo: {})
    }
}
