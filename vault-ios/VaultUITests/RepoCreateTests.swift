import VaultMobile
import XCTest

final class RepoCreateTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testCreate() async throws {
        let fixture = try await Fixture.build(createRepo: false)

        await MainActor.run {
            let app = fixture.launchApp()

            app.repoCreateWait()

            app.repoCreatePasswordFill()
            app.repoCreateAdvancedSettingsTap()
            app.repoCreateSaltFill()

            app.repoCreateCreateTap()
            app.savePasswordDismiss()

            app.repoCreateCreatedSwipeUp()

            app.repoCreateCreatedShareTap()

            app.sharePopoverDismiss()

            app.repoCreateCreatedContinueTap()

            app.repoUnlockWait()
        }
    }
}
