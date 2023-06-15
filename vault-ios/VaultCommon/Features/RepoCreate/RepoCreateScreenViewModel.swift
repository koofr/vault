import Foundation
import VaultMobile

public class RepoCreateScreenViewModel: ObservableObject {
    public let container: Container
    public let navController: MainNavController

    public let createId: UInt32

    @Published var password: String = ""
    @Published var salt: String = ""

    public init(container: Container, navController: MainNavController) {
        self.container = container
        self.navController = navController

        self.createId = container.mobileVault.repoCreateCreate()
    }

    deinit {
        container.mobileVault.repoCreateDestroy(createId: createId)
    }

    public func setLocation(mountId: String, path: String) {
        container.mobileVault.repoCreateSetLocation(
            createId: createId, location: RemoteFilesLocation(mountId: mountId, path: path))
    }

    public func setPassword(password: String) {
        if password != self.password {
            self.password = password

            container.mobileVault.repoCreateSetPassword(createId: createId, password: password)
        }
    }

    public func setSalt(salt: String) {
        if salt != self.salt {
            self.salt = salt

            container.mobileVault.repoCreateSetSalt(
                createId: createId, salt: salt.isEmpty ? nil : salt)
        }
    }

    public func fillFromRcloneConfig(config: String) -> Bool {
        return container.mobileVault.repoCreateFillFromRcloneConfig(
            createId: createId, config: config)
    }

    public func create() {
        container.mobileVault.repoCreateCreateRepo(createId: createId)
    }
}
