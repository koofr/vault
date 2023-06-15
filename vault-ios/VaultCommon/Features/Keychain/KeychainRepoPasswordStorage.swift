import Foundation
import LocalAuthentication
import Security

enum KeychainRepoPasswordStorageError: Error {
    case canceled
    case other(description: String)
}

public class KeychainRepoPasswordStorage {
    private let keychainHelper: KeychainHelper
    private let localAuthenticationHelper: LocalAuthenticationHelper

    private let access: SecAccessControl

    init(keychainHelper: KeychainHelper, localAuthenticationHelper: LocalAuthenticationHelper) {
        self.keychainHelper = keychainHelper
        self.localAuthenticationHelper = localAuthenticationHelper

        #if TARGET_OS_SIMULATOR
            let protection = kSecAttrAccessibleAfterFirstUnlock
        #else
            let protection = kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly
        #endif

        self.access = SecAccessControlCreateWithFlags(
            nil,  // Use the default allocator.
            protection,
            .biometryCurrentSet,
            nil  // Ignore any error.
        )!
    }

    public func hasPassword(repoId: String) throws -> Bool {
        var query = baseAttributes(repoId: repoId)

        localAuthenticationHelper.setContextNoInteractionContext(attributes: &query)

        query[kSecMatchLimit] = kSecMatchLimitOne

        var result: CFTypeRef?

        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status != noErr && status != errSecItemNotFound && status != errSecInteractionNotAllowed
        {
            try handleError(status: status)
        }

        return status == noErr || status == errSecInteractionNotAllowed
    }

    public func getPassword(repoId: String) throws -> String? {
        var query = baseAttributes(repoId: repoId)

        localAuthenticationHelper.setContextInteractionContext(attributes: &query)

        query[kSecMatchLimit] = kSecMatchLimitOne
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

    public func setPassword(repoId: String, password: String) throws {
        var attributes = baseAttributes(repoId: repoId)

        localAuthenticationHelper.setContextInteractionContext(attributes: &attributes)

        setPassword(attributes: &attributes, password: password)

        var status = SecItemAdd(attributes as CFDictionary, nil)

        if status == errSecDuplicateItem {
            let query = baseAttributes(repoId: repoId)

            var attributes: [CFString: Any] = [:]
            setPassword(attributes: &attributes, password: password)

            status = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        }

        try handleError(status: status)
    }

    public func removePassword(repoId: String) throws {
        var attributes = baseAttributes(repoId: repoId)

        localAuthenticationHelper.setContextNoInteractionContext(attributes: &attributes)

        let status = SecItemDelete(attributes as CFDictionary)

        try handleError(status: status)
    }

    private func baseAttributes(repoId: String) -> [CFString: Any] {
        var attrs = keychainHelper.baseAttributes()

        attrs[kSecClass] = kSecClassGenericPassword
        attrs[kSecAttrAccount] = "vaultRepoPassword_\(repoId)_v2"

        return attrs
    }

    private func setPassword(attributes: inout [CFString: Any], password: String) {
        attributes[kSecAttrLabel] = "Koofr Vault"
        attributes[kSecAttrAccessControl] = self.access
        attributes[kSecValueData] = password.data(using: .utf8)
    }

    private func handleError(status: OSStatus) throws {
        if status == noErr {
            return
        }

        if status == errSecUserCanceled {
            throw KeychainRepoPasswordStorageError.canceled
        }

        var reason: String

        let errorMessageString = SecCopyErrorMessageString(status, nil)

        if let errorMessageString = errorMessageString {
            reason = errorMessageString as String
        } else {
            reason = "Unknown Keychain error"
        }

        throw KeychainRepoPasswordStorageError.other(description: reason)
    }
}
