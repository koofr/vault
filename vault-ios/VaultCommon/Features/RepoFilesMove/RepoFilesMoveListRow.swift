import SwiftUI
import VaultMobile

public struct RepoFilesMoveListRow: View {
    @ObservedObject var vm: RepoFilesMoveScreenViewModel
    public var item: RepoFilesBrowserItem

    public var body: some View {
        let file = item.file

        HStack {
            if file.typ == .dir {
                Button {
                    vm.navController.push(
                        .repoFiles(repoId: file.repoId, encryptedPath: file.encryptedPath))
                } label: {
                    RepoFileRow(container: vm.container, file: file)
                }
            } else {
                RepoFileRow(container: vm.container, file: file)
            }
        }
    }
}
