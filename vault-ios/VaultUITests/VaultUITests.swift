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

            app.repoInfoUnlockedTap()

            app.repoInfoUnlockedAssertLocked(locked: false)
        }
    }

    func testRepoInfoAutoLockAfter() async throws {
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

    func testRepoInfoAutoLockOnAppHidden() async throws {
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

    func testRepoFilesAutoLockAfter() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp(extra: [
                "VAULT_REPOS_SET_DEFAULT_AUTO_LOCK": "3"
            ])

            app.reposRepoTap()
            app.repoUnlock()

            for _ in 1...5 {
                app.staticTexts["Folder is Empty"].tap()

                sleep(1)
            }

            sleep(5)

            app.repoUnlockWait()
        }
    }

    func testRepoFilesAutoLockOnAppHidden() async throws {
        let fixture = try await Fixture.build()

        await MainActor.run {
            let app = fixture.launchApp(extra: [
                "VAULT_REPOS_SET_DEFAULT_AUTO_LOCK": "onapphidden"
            ])

            app.reposRepoTap()
            app.repoUnlock()

            app.homeButtonPress()

            app.activate()

            app.repoUnlockWait()
        }
    }

    func testRepoFilesKeepSelectionOnLock() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "file.txt", content: "test")

        await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            app.repoFilesMenuTap()
            app.repoFilesMenuSelectTap()

            app.repoFilesEditModeWait()
            app.repoFilesFileTap(fileName: "file.txt")

            app.navigationBars["1 item"].press(forDuration: 6)

            app.repoUnlock()

            app.repoFilesEditModeWaitSelected(count: 1)
        }
    }

    func testRepoFilesDetailsTextUtf8() async throws {
        let fixture = try await Fixture.build()

        let repo = await fixture.mobileVaultHelper.waitForRepoUnlock()
        let _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "file.txt", content: "čšž")

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
            repo: repo, encryptedParentPath: "/", name: "file.jpg", content: "test")

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
