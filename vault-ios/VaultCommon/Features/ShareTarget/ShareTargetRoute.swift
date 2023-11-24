import Foundation
import SwiftUINavController
import VaultMobile

public enum ShareTargetRoute: Hashable {
    case repos
    case repoFiles(repoId: String, encryptedPath: String)
}

public typealias ShareTargetNavController = NavController<ShareTargetRoute>
