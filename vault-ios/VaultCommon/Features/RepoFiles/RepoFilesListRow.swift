import SwiftUI
import VaultMobile

public struct RepoFilesListRow: View {
    @ObservedObject var vm: RepoFilesScreenViewModel
    public var item: RepoFilesBrowserItem

    public var body: some View {
        let file = item.file

        HStack {
            if let path = file.path {
                if file.typ == .dir {
                    Button {
                        vm.navController.push(.repoFiles(repoId: file.repoId, path: path))
                    } label: {
                        RepoFileRow(container: vm.container, file: file)
                    }
                } else {
                    Button {
                        vm.navController.push(.repoFilesDetails(repoId: file.repoId, path: path))
                    } label: {
                        RepoFileRow(container: vm.container, file: file)
                    }
                }
            } else {
                RepoFileRow(container: vm.container, file: file)
            }
        }
        .contextMenu {
            RepoFileMenu(vm: vm, file: file)
        }
    }
}
