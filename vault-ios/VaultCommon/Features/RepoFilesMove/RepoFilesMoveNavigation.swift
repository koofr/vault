import SwiftUI
import SwiftUINavController
import VaultMobile

public struct RepoFilesMoveNavigation: View {
    public let vm: RepoFilesMoveViewModel

    public var body: some View {
        Navigation(navController: vm.navController) { navController, routeContainer in
            Group {
                switch routeContainer.route {
                case .repoFiles(let repoId, let path):
                    RepoGuard(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoGuardViewModel(
                                container: vm.container, repoId: repoId,
                                setupBiometricUnlockVisible: false)
                        }
                    ) {
                        RepoFilesMoveScreen(
                            vm: vm.navController.ensureViewModel(routeContainer: routeContainer) {
                                RepoFilesMoveScreenViewModel(
                                    container: vm.container, navController: navController,
                                    repoId: repoId, path: path)
                            })
                    }
                }
            }
        }
    }
}
