import VaultMobile
import XCTest

final class VaultUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testLoginCreateRevokeLoginCreate() async throws {
        let fixture = try await Fixture.build(authenticate: false, createRepo: false)

        await MainActor.run {
            let app = fixture.launchApp()

            app.landingGetStartedTap()

            app.authContinueTap()

            app.repoCreateWait()

            blockOnFatal {
                try await fixture.debugClient.oauth2Revoke()
            }

            app.repoCreateLocationTap()

            // wait for notifications to disappear
            sleep(3)

            app.landingGetStartedTap()

            app.authContinueTap()

            app.repoCreateWait()
        }
    }

    func testRepoInfoSetupBiometricsAndUnlock() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoInfoTap()

            app.repoInfoBiometricUnlockTap()

            app.repoUnlock()

            let unlocked = app.repoInfoUnlockedTap()
            XCTAssertEqual(unlocked.value as? String, "1")
        }
    }

    func testRepoCreate() async throws {
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

    func testRepoFilesMoveToNewFolder() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            app.repoFilesMenuTap()
            app.repoFilesMenuNewFolderTap()
            app.dialogsNewFolderSubmit(folderName: "Foo")

            app.repoFilesFileContextMenu(fileName: "Foo")
            app.repoFilesFileMenuMoveTap()

            app.repoFilesMoveWait()
            app.repoFilesMoveMenuTap()
            app.menuItemTap(itemName: "New folder")
            app.dialogsNewFolderSubmit(folderName: "Bar")
            app.repoFilesMoveNavigationWait(folderName: "Bar")
            app.repoFilesMoveMoveTap()
        }
    }

    func testRepoFilesEditModeDelete() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            app.repoFilesMenuTap()
            app.repoFilesMenuNewFolderTap()
            app.dialogsNewFolderSubmit(folderName: "Foo")

            app.repoFilesMenuTap()
            app.repoFilesMenuSelectTap()

            app.repoFilesEditModeWait()
            app.repoFilesFileTap(fileName: "Foo")
            app.repoFilesEditModeToolbarDeleteTap()
            app.dialogsDeleteFilesSubmit()

            app.repoFilesFileWaitNotExist(fileName: "Foo")
            app.repoFilesEditModeWaitDisabled()
        }
    }

    func testRepoFilesDetailsTextUtf8() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, parentPath: "/", name: "file.txt", content: "čšž")

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            app.repoFilesFileTap(fileName: "file.txt")

            XCTAssertTrue(app.webViews.staticTexts["čšž"].waitForExistence(timeout: 5))
        }
    }

    func testRepoFilesDetailsBackTransferAborted() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, parentPath: "/", name: "file.jpg", content: "test")

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            blockOnFatal {
                try await fixture.debugClient.downloadsPause()
            }

            app.repoFilesFileTap(fileName: "file.jpg")

            app.transfersButtonWait()

            app.repoFilesBack(parentName: "My safe box")

            app.transfersButtonWaitNotExist()
        }
    }
}
