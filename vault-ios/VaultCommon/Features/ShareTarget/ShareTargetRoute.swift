import Foundation
import VaultMobile

public enum ShareTargetRoute: Hashable {
    case repos
    case repoFiles(repoId: String, path: String)
}

public typealias ShareTargetNavController = NavController<ShareTargetRoute>
