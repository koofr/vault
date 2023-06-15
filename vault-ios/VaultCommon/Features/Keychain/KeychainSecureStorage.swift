import Foundation
import Security
import VaultMobile

public class KeychainSecureStorage: SecureStorage {
    private let keychainHelper: KeychainHelper
    private let localAuthenticationHelper: LocalAuthenticationHelper

    public init(
        keychainHelper: KeychainHelper, localAuthenticationHelper: LocalAuthenticationHelper
    ) {
        self.keychainHelper = keychainHelper
        self.localAuthenticationHelper = localAuthenticationHelper
    }

    public func getItem(key: String) throws -> String? {
        var query = baseAttributes(key: key)

        query[kSecReturnData] = true

        var result: CFTypeRef?

        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status != noErr && status != errSecItemNotFound {
            try handleError(status: status)
        }

        if result == nil {
            return nil
        }

        guard let data = result as? Data else {
            return nil
        }

        return String(decoding: data, as: UTF8.self)
    }

    public func setItem(key: String, value: String) throws {
        var attributes = baseAttributes(key: key)

        addValue(attributes: &attributes, value: value)

        var status = SecItemAdd(attributes as CFDictionary, nil)

        if status == errSecDuplicateItem {
            let query = baseAttributes(key: key)

            var attributes: [CFString: Any] = [:]
            addValue(attributes: &attributes, value: value)

            status = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        }

        try handleError(status: status)
    }

    public func removeItem(key: String) throws {
        let query = baseAttributes(key: key)

        let status = SecItemDelete(query as CFDictionary)

        try handleError(status: status)
    }

    public func clear() throws {
        var query = keychainHelper.baseAttributes()

        query[kSecClass] = kSecClassGenericPassword

        localAuthenticationHelper.setContextNoInteractionContext(attributes: &query)

        let status = SecItemDelete(query as CFDictionary)

        if status != noErr && status != errSecItemNotFound {
            try handleError(status: status)
        }
    }

    private func baseAttributes(key: String) -> [CFString: Any] {
        var attrs = keychainHelper.baseAttributes()

        attrs[kSecClass] = kSecClassGenericPassword
        attrs[kSecAttrAccount] = key

        return attrs
    }

    private func addValue(attributes: inout [CFString: Any], value: String) {
        attributes[kSecAttrLabel] = "Koofr Vault"
        attributes[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
        attributes[kSecValueData] = value.data(using: .utf8)
    }

    private func handleError(status: OSStatus) throws {
        if status == noErr {
            return
        }

        var reason: String

        let errorMessageString = SecCopyErrorMessageString(status, nil)

        if let errorMessageString = errorMessageString {
            reason = errorMessageString as String
        } else {
            reason = "Unknown Keychain error"
        }

        throw SecureStorageError.StorageError(reason: reason)
    }
}
