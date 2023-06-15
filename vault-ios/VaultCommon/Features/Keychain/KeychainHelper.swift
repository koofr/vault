import Foundation

public class KeychainHelper {
    private let service: String
    private let accessGroup: String

    public init(service: String, accessGroup: String) {
        self.service = service
        self.accessGroup = accessGroup
    }

    public func baseAttributes() -> [CFString: Any] {
        return [
            kSecAttrService: service,
            kSecAttrAccessGroup: accessGroup,
        ]
    }
}
