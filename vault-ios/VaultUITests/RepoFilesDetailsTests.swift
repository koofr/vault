import VaultMobile
import XCTest

final class RepoFilesDetailsTests: XCTestCase {
    var fixture: Fixture!
    var repo: Repo!
    var app: XCUIApplication!

    var currentContent: String!
    var serverContent: String!
    var currentName: String!
    var currentEncryptedName: String!
    var currentEncryptedParentPath: String!
    var currentEncryptedPath: String!
    var parentName: String!

    override func setUp() async throws {
        continueAfterFailure = false

        currentContent = ""
        serverContent = ""
        currentName = ""
        currentEncryptedName = ""
        currentEncryptedParentPath = ""
        currentEncryptedPath = ""
        parentName = ""

        fixture = try await Fixture.build()

        repo = await fixture.mobileVaultHelper.waitForRepoUnlock()

        app = await MainActor.run {
            let app = fixture.launchApp()

            app.reposRepoTap()
            app.repoUnlock()

            return app
        }
    }

    func navBar(name: String) -> XCUIElement {
        app.navigationBars[name]
    }

    func navBarWait(name: String) {
        XCTAssertTrue(navBar(name: name).waitForExistence(timeout: 10))
    }

    func textEditorElement() -> XCUIElement {
        return app.textViews[currentContent].firstMatch
    }

    func textEditorElementWait() -> XCUIElement {
        let element = textEditorElement()
        XCTAssertTrue(element.waitForExistence(timeout: 5))
        return element
    }

    func changeCurrentName(_ name: String) throws {
        currentName = name
        currentEncryptedName = try fixture.mobileVaultHelper.encryptName(
            repo: repo, name: currentName)
        currentEncryptedPath = joinParentName(
            parentPath: currentEncryptedParentPath, name: currentEncryptedName)
    }

    func createFile(content: String = "editorcontent") async throws {
        serverContent = content
        currentContent = serverContent
        currentEncryptedParentPath = "/"
        parentName = "My safe box"
        try changeCurrentName("file.txt")

        _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: currentName, content: currentContent)
    }

    func navigateToFile() async {
        await MainActor.run {
            app.repoFilesFileTap(fileName: currentName)
        }
    }

    func openEditor() async throws {
        await navigateToFile()
        await expectHeaderNameMatch()
        await expectCurrentContent()
    }

    func viewFile() async throws {
        try await createFile()
        try await openEditor()
    }

    func editFile() async throws {
        try await viewFile()
        await tapMoreEdit()
    }

    func goBack() async {
        await MainActor.run {
            app.navigationBars.buttons[parentName].tap()
        }
    }

    func tapDone() async {
        await MainActor.run {
            app.buttons["Done"].tap()
        }
    }

    func tapSave() async {
        await MainActor.run {
            app.buttons["Save"].tap()
        }
    }

    func tapMoreEdit() async {
        await MainActor.run {
            app.buttons.images["More"].tap()
            app.buttons["Edit"].tap()
        }
    }

    func tapMoreDelete() async {
        await MainActor.run {
            app.buttons.images["More"].tap()
            app.buttons["Delete"].tap()
        }
    }

    func editorAppendText(_ text: String) async {
        await MainActor.run {
            let element = textEditorElementWait()
            element.tap()
            element.typeText(text)
        }

        currentContent += text
    }

    func changeContent() async {
        await editorAppendText("1")
    }

    func changeContentOnServer() async {
        currentContent = "editorcontent2"
        serverContent = currentContent

        await fixture.mobileVaultHelper.setFileContent(
            repo: repo, encryptedPath: currentEncryptedPath, content: currentContent)
    }

    func renameFileOnServer() async throws {
        let name = "file renamed.txt"

        try await fixture.mobileVaultHelper.renameFile(
            repo: repo, encryptedPath: currentEncryptedPath, newName: name)

        try changeCurrentName(name)
    }

    func deleteFileOnServer() async throws {
        try await fixture.mobileVaultHelper.deleteFile(
            repo: repo, encryptedPath: currentEncryptedPath)
    }

    func reflectAutorename() throws {
        try changeCurrentName("file (1).txt")
        currentContent = "editorcontent1"
    }

    func expectHeaderNameMatch() async {
        await MainActor.run {
            navBarWait(name: currentName)
        }
    }

    func expectFileError(_ error: String) async {
        await MainActor.run {
            let element = app.staticTexts["File error"].firstMatch
            XCTAssertTrue(element.waitForExistence(timeout: 5))
            XCTAssertEqual(element.label, error)
        }
    }

    func expectCurrentContent() async {
        await MainActor.run {
            _ = textEditorElementWait()
        }
    }

    func expectNoConflicts() async {
        await MainActor.run {
            XCTAssertTrue(
                app.staticTexts["Changes are saved automatically."].firstMatch.waitForExistence(
                    timeout: 5))
        }
    }

    func expectDirty() async {
        await MainActor.run {
            XCTAssertTrue(
                app.otherElements["File modified"].firstMatch.waitForExistence(timeout: 5))
        }
    }

    func expectNotDirty() async {
        await MainActor.run {
            XCTAssertTrue(
                app.otherElements["File unchanged"].firstMatch.waitForExistence(timeout: 5))
        }
    }

    func expectFilesList() async {
        await MainActor.run {
            _ = app.repoFilesNavBarWait(name: parentName)
        }
    }

    func expectFilesListFile() async {
        await MainActor.run {
            _ = app.repoFilesFileWait(fileName: currentName)
        }
    }

    func expectFilesListFileNotExist() async {
        await MainActor.run {
            app.repoFilesFileWaitNotExist(fileName: currentName)
        }
    }

    func expectViewerOpen() async {
        await MainActor.run {
            XCTAssertTrue(app.buttons.images["More"].waitForExistence(timeout: 5))
        }
    }

    func expectServerContent(_ content: String) async throws {
        try await withTimeout(seconds: 15.0) {
            await self.fixture.mobileVaultHelper.waitForFileContent(
                repo: self.repo, encryptedPath: self.currentEncryptedPath, expectedContent: content)
        }
    }

    func expectServerContentMatch() async throws {
        try await expectServerContent(currentContent)
    }

    func handleFileNotAccessibleDialog() async {
        let alert = await app.dialogWait(
            dialogTitle: "File not accessible",
            dialogBodyRegex:
                "File \(currentName!) is no longer accessible\\. Probably it was deleted or you no longer have access to it\\.",
            primaryButtonText: "Ok")
        await alert.buttons["Ok"].tap()
    }

    func handleFileCouldNotBeSavedDialog(button: String) async {
        let alert = await app.dialogWait(
            dialogTitle: "File could not be saved",
            dialogBodyRegex:
                "File could not be saved \\(.*\\)\\. Do you want to Try again or Discard the changes\\?",
            primaryButtonText: "Try again", secondaryButtonText: "Discard changes")
        await alert.buttons[button].tap()
    }

    func handleFileWasChangedDialog(button: String, secondaryButton: String) async {
        let alert = await app.dialogWait(
            dialogTitle: "File was changed by someone else since your last save",
            dialogBodyRegex:
                "Saving into the existing file is not possible\\. Do you want to Save your changes as a new file( or Discard them)?\\?",
            primaryButtonText: "Save as a new file",
            secondaryButtonText: secondaryButton)
        await alert.buttons[button].tap()
    }

    func handleFileNotAccessibleSaveDialog(button: String, secondaryButton: String) async {
        let alert = await app.dialogWait(
            dialogTitle: "File not accessible",
            dialogBodyRegex:
                "File \(currentName!) is no longer accessible\\. Probably it was deleted or you no longer have access to it\\. Do you want to Save the file to a new location( or Discard the changes)?\\?",
            primaryButtonText: "Save to a new location",
            secondaryButtonText: secondaryButton)
        await alert.buttons[button].tap()
    }

    func handleFileLocationChangedDialog() async {
        let alert = await app.dialogWait(
            dialogTitle: "File location changed",
            dialogBodyRegex:
                "File \(currentName!) was saved here because it could not be saved in its original location\\.",
            primaryButtonText: "Ok")
        await alert.buttons["Ok"].tap()
    }

    func handleDeleteFilesDialog(button: String) async {
        let alert = await app.dialogWait(
            dialogTitle: "Delete files",
            dialogBodyRegex: "Do you really want to delete 1 item\\?",
            primaryButtonText: "Delete",
            secondaryButtonText: "Cancel")
        await alert.buttons[button].tap()
    }

    func simulateUploadError(action: @escaping () async -> Void) async throws {
        try await fixture.debugClient.withQueue { request in
            if request.url.contains("/files/put") {
                try await self.fixture.debugClient.queueNext(status: 500)

                return false
            } else {
                try await self.fixture.debugClient.queueNext()

                return true
            }
        } before: {
            await action()
        }
    }

    // Text editor: view, go back
    func testTextEditorViewGoBack() async throws {
        try await viewFile()
        await goBack()
        await expectFilesList()
    }

    // Text editor: view UTF-8
    func testTextEditorViewUtf8() async throws {
        try await createFile(content: "čšž")
        try await openEditor()
    }

    // Text editor: edit, done
    func testTextEditorEditDone() async throws {
        try await editFile()
        await tapDone()
        await expectViewerOpen()
    }

    // Text editor: edit, change, done
    func testTextEditorEditChangeDone() async throws {
        try await editFile()
        await changeContent()
        await tapDone()
        try await expectServerContentMatch()
        await expectViewerOpen()
    }

    // Text editor: edit, change, go back
    func testTextEditorEditChangeGoBack() async throws {
        try await editFile()
        await changeContent()
        await goBack()
        try await expectServerContentMatch()
        await expectFilesList()
    }

    // Text editor: edit, change, go back, error, retry, error, retry
    func testTextEditorEditChangeGoBackErrorRetryErrorRetry() async throws {
        try await editFile()
        await changeContent()

        try await simulateUploadError {
            await self.goBack()
        }

        let alert = await app.dialogWait(
            dialogTitle: "File could not be saved",
            dialogBodyRegex:
                "File could not be saved \\(.*\\)\\. Do you want to Try again or Discard the changes\\?",
            primaryButtonText: "Try again", secondaryButtonText: "Discard changes")

        try await simulateUploadError {
            await alert.buttons["Try again"].tap()
        }

        await handleFileCouldNotBeSavedDialog(button: "Try again")

        try await expectServerContentMatch()
        await expectFilesList()
    }

    // Text editor: edit, change, go back, error, discard changes
    func testTextEditorEditChangeGoBackErrorDiscardChanges() async throws {
        try await editFile()
        await changeContent()

        try await simulateUploadError {
            await self.goBack()
        }

        await handleFileCouldNotBeSavedDialog(button: "Discard changes")

        currentContent = "editorcontent"
        try await expectServerContentMatch()
        await expectFilesList()
    }

    // Text editor: edit, change, autosave
    func testTextEditorEditChangeAutosave() async throws {
        app = await MainActor.run {
            let app = fixture.launchApp(extra: [
                "VAULT_TEXT_EDITOR_AUTOSAVE_INTERVAL_MS": "1000"
            ])

            app.reposRepoTap()
            app.repoUnlock()

            return app
        }

        try await editFile()
        await changeContent()
        // content will be autosaved in 1s
        try await expectServerContentMatch()
    }

    // Text editor: edit, change, save, go back
    func testTextEditorEditChangeSaveGoBack() async throws {
        try await editFile()
        await changeContent()
        await tapSave()
        await goBack()
        try await expectServerContentMatch()
        await expectFilesList()
    }

    // Text editor: edit, change, change on server
    func testTextEditorEditChangeSaveServerChange() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await expectFileError(
            "File was changed by someone else since your last save. Automatic saving is disabled.")
    }

    // Text editor: edit, change, change on server, save, cancel
    func testTextEditorEditChangeChangeOnServerSaveCancel() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await tapSave()

        await handleFileWasChangedDialog(button: "Cancel", secondaryButton: "Cancel")
        await expectFileError(
            "File was changed by someone else since your last save. Automatic saving is disabled.")
    }

    // Text editor: edit, change, change on server, save, save as new file
    func testTextEditorEditChangeChangeOnServerSaveSaveAsNewFile() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await tapSave()

        await handleFileWasChangedDialog(button: "Save as a new file", secondaryButton: "Cancel")

        try reflectAutorename()
        try await expectServerContentMatch()
        await expectHeaderNameMatch()
        await expectNoConflicts()
        await expectNotDirty()
    }

    // Text editor: edit, change, change on server, done, discard
    func testTextEditorEditChangeChangeOnServerDoneDiscard() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await tapDone()

        await handleFileWasChangedDialog(
            button: "Discard changes", secondaryButton: "Discard changes")
        await expectViewerOpen()
        try await expectServerContentMatch()
    }

    // Text editor: edit, change, change on server, done, save as new file
    func testTextEditorEditChangeChangeOnServerDoneSaveAsNewFile() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await tapDone()

        await handleFileWasChangedDialog(
            button: "Save as a new file", secondaryButton: "Discard changes")
        try reflectAutorename()
        await expectViewerOpen()
        try await expectServerContentMatch()
        await expectHeaderNameMatch()
    }

    // Text editor: edit, change, change on server, go back, discard
    func testTextEditorEditChangeChangeOnServerGoBackDiscard() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await goBack()

        await handleFileWasChangedDialog(
            button: "Discard changes", secondaryButton: "Discard changes")
        await expectFilesList()
    }

    // Text editor: edit, change, change on server, go back, save as new file
    func testTextEditorEditChangeChangeOnServerGoBackSaveAsNewFile() async throws {
        try await editFile()
        await changeContent()
        await changeContentOnServer()
        await goBack()

        await handleFileWasChangedDialog(
            button: "Save as a new file", secondaryButton: "Discard changes")
        try reflectAutorename()
        await expectFilesList()
        await expectFilesListFile()
        try await expectServerContentMatch()
    }

    // Text editor: view, change on server, reloaded
    func testTextEditorViewChangeOnServerReloaded() async throws {
        try await viewFile()
        await changeContentOnServer()
        await expectCurrentContent()
    }

    // Text editor: edit, change on server, reloaded
    func testTextEditorEditChangeOnServerReloaded() async throws {
        try await editFile()
        await changeContentOnServer()
        await expectCurrentContent()
        await expectNoConflicts()
        await expectNotDirty()
    }

    // Text editor: view, rename on server
    func testTextEditorViewRenameOnServer() async throws {
        try await viewFile()
        try await renameFileOnServer()
        await expectHeaderNameMatch()
        await expectCurrentContent()
    }

    // Text editor: edit, rename on server
    func testTextEditorEditRenameOnServer() async throws {
        try await editFile()
        try await renameFileOnServer()
        await expectHeaderNameMatch()
        await expectCurrentContent()
        await expectNoConflicts()
        await expectNotDirty()
    }

    // Text editor: edit, change, rename on server
    func testTextEditorEditChangeRenameOnServer() async throws {
        try await editFile()
        await changeContent()
        try await renameFileOnServer()
        await expectHeaderNameMatch()
        await expectDirty()
    }

    // Text editor: edit, change, rename on server, save
    func testTextEditorEditChangeRenameOnServerSave() async throws {
        try await editFile()
        await changeContent()
        try await renameFileOnServer()
        await expectHeaderNameMatch()
        await expectDirty()
        await tapSave()
        await expectNotDirty()
        try await expectServerContentMatch()
    }

    // Text editor: view, delete, cancel
    func testTextEditorViewDeleteCancel() async throws {
        try await viewFile()
        await tapMoreDelete()

        await handleDeleteFilesDialog(button: "Cancel")

        await expectViewerOpen()
        await expectCurrentContent()
    }

    // Text editor: view, delete, confirm
    func testTextEditorViewDeleteConfirm() async throws {
        try await viewFile()
        await tapMoreDelete()

        await handleDeleteFilesDialog(button: "Delete")

        await expectFilesList()
        await expectFilesListFileNotExist()
    }

    // Text editor: edit, change, delete on server
    func testTextEditorEditChangeDeleteOnServer() async throws {
        try await editFile()
        await changeContent()
        try await deleteFileOnServer()

        await handleFileNotAccessibleDialog()

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )
    }

    // Text editor: edit, change, delete on server, save, cancel
    func testTextEditorEditChangeDeleteOnServerSaveCancel() async throws {
        try await editFile()
        await changeContent()
        try await deleteFileOnServer()

        await handleFileNotAccessibleDialog()

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )

        await tapSave()

        await handleFileNotAccessibleSaveDialog(button: "Cancel", secondaryButton: "Cancel")

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )
    }

    // Text editor: edit, change, delete on server, save, save to new location
    func testTextEditorEditChangeDeleteOnServerSaveSaveToNewLocation() async throws {
        try await editFile()
        await changeContent()
        try await deleteFileOnServer()

        await handleFileNotAccessibleDialog()

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )

        await tapSave()

        await handleFileNotAccessibleSaveDialog(
            button: "Save to a new location", secondaryButton: "Cancel")

        await handleFileLocationChangedDialog()

        await expectFilesListFile()
    }

    // Text editor: edit, change, delete on server, go back, discard changes
    func testTextEditorEditChangeDeleteOnServerGoBackDiscardChanges() async throws {
        try await editFile()
        await changeContent()
        try await deleteFileOnServer()

        await handleFileNotAccessibleDialog()

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )

        await goBack()

        await handleFileNotAccessibleSaveDialog(
            button: "Discard changes", secondaryButton: "Discard changes")

        await expectFilesList()
        await expectFilesListFileNotExist()
    }

    // Text editor: edit, change, delete on server, go back, save to new location
    func testTextEditorEditChangeDeleteOnServerGoBackSaveToNewLocation() async throws {
        try await editFile()
        await changeContent()
        try await deleteFileOnServer()

        await handleFileNotAccessibleDialog()

        await expectFileError(
            "This file is no longer accessible. Probably it was deleted or you no longer have access to it."
        )

        await goBack()

        await handleFileNotAccessibleSaveDialog(
            button: "Save to a new location", secondaryButton: "Discard changes")

        await handleFileLocationChangedDialog()

        await expectFilesListFile()
    }

    func testPdfViewer() async throws {
        let examplePdf = try Data(
            contentsOf: Bundle(for: type(of: self)).url(
                forResource: "example", withExtension: "pdf")!)
        _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "example.pdf", data: examplePdf)

        await MainActor.run {
            app.repoFilesFileTap(fileName: "example.pdf")

            XCTAssertTrue(app.webViews.firstMatch.waitForExistence(timeout: 5))

            // assert that pdf is really rendered
            XCTAssertTrue(app.webViews.staticTexts["1 of 1"].waitForExistence(timeout: 5))
        }
    }

    func testBackTransferAborted() async throws {
        _ = await fixture.mobileVaultHelper.uploadFile(
            repo: repo, encryptedParentPath: "/", name: "file.jpg", content: "test")

        try await fixture.debugClient.downloadsPause()

        await MainActor.run {
            app.repoFilesFileTap(fileName: "file.jpg")

            app.transfersButtonWait()

            app.repoFilesBack(parentName: "My safe box")

            app.transfersButtonWaitNotExist()
        }
    }
}
