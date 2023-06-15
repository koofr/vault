import Foundation

public class MemorySecureStorage: SecureStorage {
    private var data: [String: String]

    public init() {
        self.data = [String: String]()
    }

    public func getData() -> [String: String] {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }

        return data
    }

    public func getItem(key: String) throws -> String? {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }

        return data[key]
    }

    public func setItem(key: String, value: String) throws {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }

        data[key] = value
    }

    public func removeItem(key: String) throws {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }

        data.removeValue(forKey: key)
    }

    public func clear() throws {
        objc_sync_enter(self)
        defer { objc_sync_exit(self) }

        data.removeAll()
    }
}
