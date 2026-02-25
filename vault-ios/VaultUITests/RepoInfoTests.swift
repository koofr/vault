import VaultMobile
import XCTest

final class RepoInfoTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testSetupBiometricsAndUnlock() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoInfoTap()

            app.repoInfoBiometricUnlockTap()

            app.repoUnlock()

            app.repoInfoLockedTap()

            app.repoInfoAssertUnlocked()
        }
    }

    func testAutoLockAfter() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoInfoTap()

            app.reposRepoInfoLockAfterTap()

            app.reposRepoInfoLockAfterChoiceTap(choice: "10 minutes of inactivity")

            app.reposRepoInfoBackTap()

            app.reposRepoInfoTap()

            let _ = app.reposRepoInfoLockAfterWait(after: "10 minutes of inactivity")
        }
    }

    func testAutoLockOnAppHidden() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoInfoTap()

            app.reposRepoInfoLockOnAppHiddenTap()

            app.reposRepoInfoBackTap()

            app.reposRepoInfoTap()

            app.repoInfoUnlockedExpectEnabled(enabled: true)
        }
    }

}
