import Foundation
import SwiftUINavController

public enum MainRoute: Hashable {
    case repos
    case repoFiles(repoId: String, path: String)
    case repoFilesDetails(repoId: String, path: String)
    case repoInfo(repoId: String)
    case repoRemove(repoId: String)
    case repoCreate
}

public typealias MainNavController = NavController<MainRoute>
