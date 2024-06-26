import SwiftUI
import SwiftUINavController
import VaultMobile

public struct ShareTargetNavigation: View {
    public let vm: ShareTargetViewModel

    public init(vm: ShareTargetViewModel) {
        self.vm = vm
    }

    public var body: some View {
        Navigation(navController: vm.navController) { navController, routeContainer in
            Group {
                switch routeContainer.route {
                case .repos:
                    ShareTargetReposScreen(vm: vm)
                case .repoFiles(let repoId, let encryptedPath):
                    RepoGuard(
                        navController.ensureViewModel(routeContainer: routeContainer) {
                            ShareTargetRepoFilesScreenViewModel(
                                container: vm.container, shareTargetVm: vm,
                                repoId: repoId, encryptedPath: encryptedPath)
                        }
                    ) { vm in
                        ShareTargetRepoFilesScreen(vm: vm)
                    }
                }
            }
        }
    }
}
