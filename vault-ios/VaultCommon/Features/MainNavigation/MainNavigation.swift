import SwiftUI
import SwiftUINavController

public struct MainNavigation: View {
    public let container: Container
    public let navController: MainNavController

    public init(container: Container, navController: MainNavController) {
        self.container = container
        self.navController = navController
    }

    public var body: some View {
        Navigation(navController: navController) { navController, routeContainer in
            Group {
                switch routeContainer.route {
                case .repos:
                    ReposScreen(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            ReposScreenViewModel(container: container, navController: navController)
                        })
                case .repoFiles(let repoId, let path):
                    RepoGuard(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoGuardViewModel(
                                container: container, repoId: repoId,
                                setupBiometricUnlockVisible: true)
                        }
                    ) {
                        RepoFilesScreen(
                            vm: navController.ensureViewModel(routeContainer: routeContainer) {
                                RepoFilesScreenViewModel(
                                    container: container, navController: navController,
                                    repoId: repoId, path: path)
                            })
                    }
                case .repoFilesDetails(let repoId, let path):
                    RepoGuard(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoGuardViewModel(
                                container: container, repoId: repoId,
                                setupBiometricUnlockVisible: true)
                        }
                    ) {
                        RepoFilesDetailsScreen(
                            vm: navController.ensureViewModel(routeContainer: routeContainer) {
                                RepoFilesDetailsScreenViewModel(
                                    container: container, repoId: repoId, path: path)
                            })
                    }
                case .repoInfo(let repoId):
                    RepoInfoScreen(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoInfoScreenViewModel(
                                container: container, navController: navController, repoId: repoId)
                        })
                case .repoRemove(let repoId):
                    RepoRemoveScreen(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoRemoveScreenViewModel(
                                container: container, navController: navController, repoId: repoId)
                        })
                case .repoCreate:
                    RepoCreateScreen(
                        vm: navController.ensureViewModel(routeContainer: routeContainer) {
                            RepoCreateScreenViewModel(
                                container: container, navController: navController)
                        })
                }
            }
        }
    }
}
