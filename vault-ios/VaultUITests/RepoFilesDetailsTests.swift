import VaultMobile
import XCTest

final class RepoFilesDetailsTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testTextUtf8() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "file.txt", content: "čšž")

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            app.repoFilesFileTap(fileName: "file.txt")

            XCTAssertTrue(
                app.textViews.matching(identifier: "čšž").firstMatch.waitForExistence(timeout: 5))
        }
    }

    func testBackTransferAborted() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "file.jpg", content: "test")

        let app = await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            return app
        }

        try await fixture.debugClient.downloadsPause()

        await MainActor.run {
            app.repoFilesFileTap(fileName: "file.jpg")

            app.transfersButtonWait()

            app.repoFilesBack(parentName: "My safe box")

            app.transfersButtonWaitNotExist()
        }
    }
}
