import Foundation

public class StorageHelper {
    public init() {}

    public func getTempDir() -> URL {
        return FileManager.default.temporaryDirectory
    }

    public func getDownloadsDir() throws -> URL {
        return try FileManager.default.url(
            for: .documentDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    }

    public func clearCache() throws {
        let url = getTempDir()

        let children = try FileManager.default.contentsOfDirectory(
            at: url, includingPropertiesForKeys: nil)

        for childURL in children {
            try FileManager.default.removeItem(at: childURL)
        }
    }
}
