import Foundation
import VaultMobile

public class Container: ObservableObject {
    public let baseURL: String
    public let localAuthenticationHelper: LocalAuthenticationHelper
    public let keychainHelper: KeychainHelper
    public let keychainSecureStorage: KeychainSecureStorage
    public let mobileVault: MobileVault
    public let keychainRepoPasswordStorage: KeychainRepoPasswordStorage
    public let authHelper: AuthHelper
    public let fileIconCache: FileIconCache
    public var sheets: Sheets
    public let transfersSheetController: TransfersSheetController
    public let repoFilesMoveSheetController: RepoFilesMoveSheetController
    public let storageHelper: StorageHelper
    public let downloadHelper: DownloadHelper
    public let uploadHelper: UploadHelper

    public init(
        baseURL: String? = nil, oauth2AuthBaseURL: String? = nil, secureStorageJson: String? = nil
    ) {
        let baseURL = baseURL ?? "https://app.koofr.net"
        let appName = "vault-ios"
        let oauth2AuthBaseURL = oauth2AuthBaseURL ?? baseURL
        let oauth2ClientId = "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3"
        let oauth2ClientSecret = "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB"
        let oauth2RedirectUri = "koofrvault://oauth2callback"

        let keychainService = "net.koofr.Vault"
        let keychainAccessGroup = "group.net.koofr.Vault"

        self.baseURL = baseURL

        localAuthenticationHelper = LocalAuthenticationHelper()
        keychainHelper = KeychainHelper(service: keychainService, accessGroup: keychainAccessGroup)
        keychainSecureStorage = KeychainSecureStorage(
            keychainHelper: keychainHelper, localAuthenticationHelper: localAuthenticationHelper)

        if let secureStorageJson = secureStorageJson {
            do {
                let items = try JSONDecoder().decode(
                    [String: String].self, from: secureStorageJson.data(using: .utf8)!)

                try keychainSecureStorage.clear()

                for (key, value) in items {
                    try keychainSecureStorage.setItem(key: key, value: value)
                }
            } catch {
                print("KeychainSecureStorage init error: \(error)")
            }
        }

        mobileVault = MobileVault(
            baseUrl: baseURL, appName: appName, oauth2AuthBaseUrl: oauth2AuthBaseURL,
            oauth2ClientId: oauth2ClientId, oauth2ClientSecret: oauth2ClientSecret,
            oauth2RedirectUri: oauth2RedirectUri, secureStorage: keychainSecureStorage)

        mobileVault.load()

        keychainRepoPasswordStorage = KeychainRepoPasswordStorage(
            keychainHelper: keychainHelper, localAuthenticationHelper: localAuthenticationHelper)

        authHelper = AuthHelper(mobileVault: mobileVault, baseURL: baseURL)

        fileIconCache = FileIconCache(mobileVault: mobileVault)

        sheets = Sheets()

        transfersSheetController = TransfersSheetController()

        repoFilesMoveSheetController = RepoFilesMoveSheetController()

        storageHelper = StorageHelper()

        downloadHelper = DownloadHelper(
            mobileVault: mobileVault, sheets: sheets,
            transfersSheetController: transfersSheetController, storageHelper: storageHelper)

        uploadHelper = UploadHelper(mobileVault: mobileVault, storageHelper: storageHelper)

        transfersSheetController.setContainer(container: self)

        repoFilesMoveSheetController.setContainer(container: self)
    }
}
