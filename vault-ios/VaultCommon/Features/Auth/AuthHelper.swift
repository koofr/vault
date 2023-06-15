import AuthenticationServices
import VaultMobile

public class AuthHelper: NSObject, ASWebAuthenticationPresentationContextProviding {
    private let mobileVault: MobileVault
    private let baseURL: String

    public init(mobileVault: MobileVault, baseURL: String) {
        self.mobileVault = mobileVault
        self.baseURL = baseURL
    }

    public func login(onDone: @escaping () -> Void) {
        if let url = mobileVault.oauth2StartLoginFlow() {
            startFlow(startFlowURL: "\(url)&platform=ios", onDone: onDone)
        }
    }

    public func logout(onDone: @escaping () -> Void) {
        if let url = mobileVault.oauth2StartLogoutFlow() {
            startFlow(startFlowURL: "\(url)&platform=ios", onDone: onDone)
        }
    }

    public func removeAccount() {
        let url = "\(baseURL)/app/remove-account?platform=ios"

        startFlow(
            startFlowURL: url,
            onDone: {
                // try to load. if the account has been removed, load will fail
                // and user will be redirected to the landing page
                self.mobileVault.load()
            })
    }

    public func startFlow(startFlowURL: String, onDone: @escaping () -> Void) {
        let startFlowURL = URL(string: startFlowURL)!

        let scheme = "koofrvault"

        let windowSubviewRemoved = WindowSubviewRemoved()

        let session = ASWebAuthenticationSession(url: startFlowURL, callbackURLScheme: scheme) {
            url, err in
            if let err = err {
                onDone()

                if case ASWebAuthenticationSessionError.canceledLogin = err {
                    return
                } else {
                    self.mobileVault.notificationsShow(message: "Authentication error: \(err)")
                }
            } else if let url = url {
                // we need to wait for ASWebAuthenticationSession's sheet to disappear because
                // otherwise it breaks NavigationStack if we navigate when the sheet is still
                // visible
                windowSubviewRemoved.start(fallbackDuration: .seconds(2)) {
                    self.mobileVault.oauth2FinishFlowUrl(
                        url: url.absoluteString, cb: OAuth2FinishFlowDoneFn(onDone))
                }
            }
        }

        session.presentationContextProvider = self

        session.prefersEphemeralWebBrowserSession = false

        session.start()
    }

    public func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor
    {
        return ASPresentationAnchor()
    }
}
