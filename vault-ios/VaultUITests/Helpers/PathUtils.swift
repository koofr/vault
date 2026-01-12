import Foundation

enum PathError: Error {
    case rootPathNotAllowed
    case invalidPath(String)
}

func joinParentName(parentPath: String, name: String) -> String {
    parentPath == "/" ? "/\(name)" : "\(parentPath)/\(name)"
}

func splitParentName(_ path: String) throws -> (String, String) {
    if path == "/" {
        throw PathError.rootPathNotAllowed
    }

    var parts = path.split(separator: "/", omittingEmptySubsequences: false)
        .map(String.init)

    guard let name = parts.popLast(), !name.isEmpty else {
        throw PathError.invalidPath(path)
    }

    var parentPath = parts.joined(separator: "/")
    if parentPath.isEmpty {
        parentPath = "/"
    }

    return (parentPath, name)
}
