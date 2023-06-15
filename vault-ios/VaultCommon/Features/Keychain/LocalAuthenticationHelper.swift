import Foundation
import LocalAuthentication

public class LocalAuthenticationHelper {
    public init() {
    }

    public func noInteractionContext() -> LAContext {
        let context = LAContext()

        context.interactionNotAllowed = true

        return context
    }

    public func interactionContext() -> LAContext {
        let context = LAContext()

        context.localizedReason = "Access your Safe Key on the keychain"

        return context
    }

    func setContext(attributes: inout [CFString: Any], context: LAContext) {
        attributes[kSecUseAuthenticationContext] = context
    }

    func setContextNoInteractionContext(attributes: inout [CFString: Any]) {
        setContext(attributes: &attributes, context: noInteractionContext())
    }

    func setContextInteractionContext(attributes: inout [CFString: Any]) {
        setContext(attributes: &attributes, context: interactionContext())
    }
}
