import Foundation
import SwiftUINavController

public enum MainRoute: Hashable {
    case repos
    case repoFiles(repoId: String, encryptedPath: String)
    case repoFilesDetails(repoId: String, encryptedPath: String)
    case repoInfo(repoId: String)
    case repoRemove(repoId: String)
    case repoCreate
}

public typealias MainNavController = NavController<MainRoute>
