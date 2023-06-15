import Combine
import SwiftUI
import VaultMobile

public class RepoFilesMoveViewModel: ObservableObject {
    public let container: Container

    @ObservedObject var navController: RepoFilesMoveNavController

    private var pathChangedCancellable: AnyCancellable?

    public init(container: Container, repoId: String, initialPathChain: [String]) {
        self.container = container

        navController = NavController(rootRoute: .repoFiles(repoId: repoId, path: "/"))

        pathChangedCancellable = navController.$state.sink(receiveValue: { state in
            switch state.activeRoute {
            case .repoFiles(_, let path):
                container.mobileVault.repoFilesMoveSetDestPath(destPath: path)
            }
        })

        for path in initialPathChain {
            if path != "/" {
                navController.push(.repoFiles(repoId: repoId, path: path))
            }
        }
    }
}
