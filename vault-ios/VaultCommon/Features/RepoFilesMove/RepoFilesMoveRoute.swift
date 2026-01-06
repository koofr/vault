import Foundation
import SwiftUINavController
import VaultMobile

public enum RepoFilesMoveRoute: Hashable {
    case repoFiles(repoId: String, encryptedPath: String)
}

public typealias RepoFilesMoveNavController = NavController<RepoFilesMoveRoute>
public typealias RepoFilesMoveVaultNavController = VaultNavController<RepoFilesMoveRoute>
