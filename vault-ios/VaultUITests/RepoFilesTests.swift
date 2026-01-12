import VaultMobile
import XCTest

final class RepoFilesTests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
    }

    func testMoveToNewFolder() async throws {
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

    func testEditModeDelete() async throws {
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

    func testAutoLockAfter() async throws {
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

    func testAutoLockOnAppHidden() async throws {
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

    func testKeepSelectionOnLock() async throws {
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
}
