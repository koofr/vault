import Combine
import SwiftUI
import VaultMobile

public class RepoFilesMoveViewModel: ObservableObject {
    public let container: Container

    @ObservedObject var navController: RepoFilesMoveVaultNavController

    private var pathChangedCancellable: AnyCancellable?

    public init(container: Container, repoId: String, initialEncryptedPathChain: [String]) {
        self.container = container

        navController = RepoFilesMoveVaultNavController(
            navController: RepoFilesMoveNavController(
                rootRoute: .repoFiles(repoId: repoId, encryptedPath: "/")),
            mobileVault: container.mobileVault)

        pathChangedCancellable = navController.navController.$state.sink(receiveValue: { state in
            switch state.activeRoute {
            case .repoFiles(_, let encryptedPath):
                container.mobileVault.repoFilesMoveSetDestPath(encryptedDestPath: encryptedPath)
            }
        })

        for encryptedPath in initialEncryptedPathChain {
            if encryptedPath != "/" {
                navController.navController.push(
                    .repoFiles(repoId: repoId, encryptedPath: encryptedPath))
            }
        }
    }
}
