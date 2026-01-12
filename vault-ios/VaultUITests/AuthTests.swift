import VaultMobile
import XCTest

final class AuthTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testLoginCreateRevokeLoginCreate() async throws {
        let fixture = try await Fixture.build(authenticate: false, createRepo: false)

        let app = await MainActor.run {
            let app = fixture.launchApp()

            app.landingGetStartedTap()

            app.authContinueTap()

            app.repoCreateWait()

            return app
        }

        try await fixture.debugClient.oauth2Revoke()

        await MainActor.run {
            app.repoCreateLocationTap()

            // wait for notifications to disappear
            sleep(3)

            app.landingGetStartedTap()

            app.authContinueTap()

            app.repoCreateWait()
        }
    }
}
