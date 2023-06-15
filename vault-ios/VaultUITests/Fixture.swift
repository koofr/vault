import VaultMobile
import XCTest

class Fixture {
    let fakeRemote: FakeRemote
    let baseUrl: String
    let oauth2AuthBaseUrl: String
    let debugClient: DebugClient
    let mobileVault: MobileVault
    let mobileVaultHelper: MobileVaultHelper
    let secureStorageJson: String

    init(
        fakeRemote: FakeRemote, baseUrl: String, oauth2AuthBaseUrl: String,
        debugClient: DebugClient, mobileVault: MobileVault, mobileVaultHelper: MobileVaultHelper,
        secureStorageJson: String
    ) {
        self.fakeRemote = fakeRemote
        self.baseUrl = baseUrl
        self.oauth2AuthBaseUrl = oauth2AuthBaseUrl
        self.debugClient = debugClient
        self.mobileVault = mobileVault
        self.mobileVaultHelper = mobileVaultHelper
        self.secureStorageJson = secureStorageJson
    }

    static func build(authenticate: Bool = true, createRepo: Bool = true) async throws -> Fixture {
        let fakeRemote = try FakeRemote(httpAddr: "127.0.0.1:0", httpsAddr: "127.0.0.1:0")

        let started = try fakeRemote.start()

        let httpURL = started.httpUrl
        let httpsURL = started.httpsUrl

        //let httpUrl = "http://127.0.0.1:3080"
        //let httpsUrl = "https://127.0.0.1:3443"

        print("FakeRemote HTTP URL: \(httpURL)")
        print("FakeRemote HTTPS URL: \(httpsURL)")

        let baseUrl = httpsURL
        let oauth2AuthBaseUrl = httpURL
        let oauth2ClientId = "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3"
        let oauth2ClientSecret = "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB"
        let oauth2RedirectUri = "koofrvault://oauth2callback"
        let debugBaseUrl = httpURL

        let debugClient = DebugClient(baseUrl: debugBaseUrl)

        try await debugClient.reset()

        if createRepo {
            try await debugClient.createTestVaultRepo()
        }

        let memorySecureStorage = MemorySecureStorage()

        if authenticate {
            try memorySecureStorage.setItem(
                key: "vaultOAuth2Token",
                value:
                    "{\"access_token\":\"\",\"refresh_token\":\"a126768a-ce0b-4b93-8a9b-809f02f4c000\",\"expires_at\":0}"
            )
        }

        let mobileVault = MobileVault(
            baseUrl: baseUrl, appName: "vault-ios-tests", oauth2AuthBaseUrl: oauth2AuthBaseUrl,
            oauth2ClientId: oauth2ClientId,
            oauth2ClientSecret: oauth2ClientSecret, oauth2RedirectUri: oauth2RedirectUri,
            secureStorage: memorySecureStorage)
        let mobileVaultHelper = MobileVaultHelper(mobileVault: mobileVault)

        mobileVault.load()

        if authenticate {
            await mobileVaultHelper.waitForOAuth2Loaded()
            await mobileVaultHelper.waitForReposLoaded()
        }

        let secureStorageJson = try String(
            decoding: JSONEncoder().encode(memorySecureStorage.getData()), as: UTF8.self)

        return Fixture(
            fakeRemote: fakeRemote, baseUrl: baseUrl,
            oauth2AuthBaseUrl: oauth2AuthBaseUrl, debugClient: debugClient,
            mobileVault: mobileVault, mobileVaultHelper: mobileVaultHelper,
            secureStorageJson: secureStorageJson)
    }

    deinit {
        do {
            try fakeRemote.stop()
        } catch {
            print("Failed to stop fake remote: \(error)")
        }
    }

    func launchApp() -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["VAULT_BASE_URL"] = baseUrl
        app.launchEnvironment["VAULT_OAUTH2_AUTH_BASE_URL"] = oauth2AuthBaseUrl
        app.launchEnvironment["VAULT_SECURE_STORAGE"] = secureStorageJson

        app.launch()

        return app
    }
}
