package net.koofr.vault.tests

import android.widget.EditText
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import net.koofr.vault.Repo
import net.koofr.vault.tests.helpers.Fixture
import net.koofr.vault.tests.helpers.UIHelpers
import net.koofr.vault.tests.helpers.joinParentName
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

// tested on Medium Phone API 36.1
@RunWith(AndroidJUnit4::class)
class RepoFilesDetailsTest {
    private lateinit var fixture: Fixture
    private lateinit var device: UiDevice
    private lateinit var h: UIHelpers
    private lateinit var repo: Repo

    private lateinit var currentContent: String
    private lateinit var serverContent: String
    private lateinit var currentName: String
    private lateinit var currentEncryptedName: String
    private lateinit var currentEncryptedParentPath: String
    private lateinit var currentEncryptedPath: String
    private lateinit var parentName: String

    @Before
    fun setUp() {
        currentContent = ""
        serverContent = ""
        currentName = ""
        currentEncryptedName = ""
        currentEncryptedParentPath = ""
        currentEncryptedPath = ""
        parentName = ""

        fixture = Fixture.build(authenticate = true, createRepo = true)
        device = fixture.launchApp()
        h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        repo = fixture.mobileVaultHelper.waitForRepoUnlock()
    }

    @After
    fun tearDown() {
        fixture.close()
    }

    private fun navBarWait(name: String) {
        checkNotNull(
            device.wait(Until.findObject(By.text(name)), 10000),
        ) { "Nav bar '$name' was not found" }
    }

    private fun textEditorElementWait() {
        h.repoFilesDetailsTextEditorContentWait(currentContent)
    }

    private fun changeCurrentName(name: String) {
        currentName = name
        currentEncryptedName = fixture.mobileVaultHelper.encryptName(repo, currentName)
        currentEncryptedPath = joinParentName(currentEncryptedParentPath, currentEncryptedName)
    }

    private fun createFile(content: String = "editorcontent") {
        serverContent = content
        currentContent = serverContent
        currentEncryptedParentPath = "/"
        parentName = "My safe box"
        changeCurrentName("file.txt")

        fixture.mobileVaultHelper.uploadFile(repo, "/", currentName, currentContent)
    }

    private fun navigateToFile() {
        h.repoFilesFileRowClick(currentName)
    }

    private fun openEditor() {
        navigateToFile()
        expectHeaderNameMatch()
        expectCurrentContent()
    }

    private fun viewFile() {
        createFile()
        openEditor()
    }

    private fun editFile() {
        viewFile()
        clickMoreEdit()
    }

    private fun goBack() {
        device.waitForIdle()
        device.pressBack()
        device.waitForIdle()
    }

    private fun clickDone() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("Done")), 10000),
        ) { "Done button was not found" }.click()
    }

    private fun clickSave() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("Save")), 10000),
        ) { "Save button was not found" }.click()
    }

    private fun clickMoreEdit() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("More")), 10000),
        ) { "More button was not found" }.click()
        h.menuItemClick("Edit")
    }

    private fun clickMoreDelete() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("More")), 10000),
        ) { "More button was not found" }.click()
        h.menuItemClick("Delete")
    }

    val editorTextSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("File text editor"))

    private fun editorTextElement(): UiObject2 {
        val element = checkNotNull(
            device.wait(Until.findObject(editorTextSelector), 10000),
        ) { "Text editor was not found" }
        return element
    }

    private fun editorAppendText(text: String) {
        val element = editorTextElement()
        element.click()
        val current = element.text ?: ""
        element.text = current + text
        currentContent += text

        // press back to stop editing text
        device.pressBack()
    }

    private fun changeContent() {
        editorAppendText("1")
    }

    private fun changeContentOnServer() {
        currentContent = "editorcontent2"
        serverContent = currentContent

        fixture.mobileVaultHelper.setFileContent(repo, currentEncryptedPath, currentContent)
    }

    private fun renameFileOnServer() {
        val name = "file renamed.txt"

        fixture.mobileVaultHelper.renameFile(repo, currentEncryptedPath, name)

        changeCurrentName(name)
    }

    private fun deleteFileOnServer() {
        fixture.mobileVaultHelper.deleteFile(repo, currentEncryptedPath)
    }

    private fun reflectAutorename() {
        changeCurrentName("file (1).txt")
        currentContent = "editorcontent1"
    }

    private fun expectHeaderNameMatch() {
        navBarWait(currentName)
    }

    private fun expectFileError(error: String) {
        checkNotNull(
            device.wait(Until.findObject(By.desc("File error").text(error)), 10000),
        ) { "File error '$error' was not found" }
    }

    private fun expectCurrentContent() {
        textEditorElementWait()
    }

    private fun expectNoConflicts() {
        checkNotNull(
            device.wait(Until.findObject(By.text("Changes are saved automatically.")), 10000),
        ) { "No conflicts message was not found" }
    }

    private fun expectDirty() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("File modified")), 10000),
        ) { "File modified indicator was not found" }
    }

    private fun expectNotDirty() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("File unchanged")), 10000),
        ) { "File unchanged indicator was not found" }
    }

    private fun expectFilesList() {
        navBarWait(parentName)
    }

    private fun expectFilesListFile() {
        h.repoFilesFileRowWait(currentName)
    }

    private fun expectFilesListFileNotExist() {
        h.repoFilesFileRowWaitNotExist(currentName)
    }

    private fun expectViewerOpen() {
        checkNotNull(
            device.wait(Until.findObject(By.desc("More")), 10000),
        ) { "More icon was not found" }
    }

    private fun expectServerContent(content: String) {
        fixture.mobileVaultHelper.waitForFileContent(repo, currentEncryptedPath, content)
    }

    private fun expectServerContentMatch() {
        expectServerContent(currentContent)
    }

    private fun handleFileNotAccessibleDialog() {
        h.dialogWait(
            dialogTitle = "File not accessible",
            dialogBodyRegex = "File $currentName is no longer accessible\\. Probably it was deleted or you no longer have access to it\\.",
            primaryButtonText = "Ok",
        )
        h.dialogButtonClick("Ok")
    }

    private fun handleFileCouldNotBeSavedDialog(button: String) {
        h.dialogWait(
            dialogTitle = "File could not be saved",
            dialogBodyRegex = "File could not be saved .*\\. Do you want to Try again or Discard the changes\\?",
            primaryButtonText = "Try again",
            secondaryButtonText = "Discard changes",
        )
        h.dialogButtonClick(button)
    }

    private fun handleFileWasChangedDialog(button: String, secondaryButton: String) {
        h.dialogWait(
            dialogTitle = "File was changed by someone else since your last save",
            dialogBodyRegex = "Saving into the existing file is not possible\\. Do you want to Save your changes as a new file( or Discard them)?\\?",
            primaryButtonText = "Save as a new file",
            secondaryButtonText = secondaryButton,
        )
        h.dialogButtonClick(button)
    }

    private fun handleFileNotAccessibleSaveDialog(button: String, secondaryButton: String) {
        h.dialogWait(
            dialogTitle = "File not accessible",
            dialogBodyRegex = "File $currentName is no longer accessible\\. Probably it was deleted or you no longer have access to it\\. Do you want to Save the file to a new location( or Discard the changes)?\\?",
            primaryButtonText = "Save to a new location",
            secondaryButtonText = secondaryButton,
        )
        h.dialogButtonClick(button)
    }

    private fun handleFileLocationChangedDialog() {
        h.dialogWait(
            dialogTitle = "File location changed",
            dialogBodyRegex = "File $currentName was saved here because it could not be saved in its original location\\.",
            primaryButtonText = "Ok",
        )
        h.dialogButtonClick("Ok")
    }

    private fun handleDeleteFilesDialog(button: String) {
        h.dialogWait(
            dialogTitle = "Delete files",
            dialogBodyRegex = "Do you really want to delete 1 item\\?",
            primaryButtonText = "Delete",
            secondaryButtonText = "Cancel",
        )
        h.dialogButtonClick(button)
    }

    private fun simulateUploadError(action: () -> Unit) {
        fixture.debugClient.withQueue({ request ->
            if (request.url.contains("/files/put")) {
                fixture.debugClient.queueNext(500)
                false
            } else {
                fixture.debugClient.queueNext()
                true
            }
        }, action)
    }

    // Text editor: view, go back
    @Test
    fun testTextEditorViewGoBack() {
        viewFile()
        goBack()
        expectFilesList()
    }

    // Text editor: view UTF-8
    @Test
    fun testTextEditorViewUtf8() {
        createFile("čšž")
        openEditor()
    }

    // Text editor: edit, done
    @Test
    fun testTextEditorEditDone() {
        editFile()
        clickDone()
        expectViewerOpen()
    }

    // Text editor: edit, change, done
    @Test
    fun testTextEditorEditChangeDone() {
        editFile()
        changeContent()
        clickDone()
        expectServerContentMatch()
        expectViewerOpen()
    }

    // Text editor: edit, change, go back
    @Test
    fun testTextEditorEditChangeGoBack() {
        editFile()
        changeContent()
        goBack()
        expectServerContentMatch()
        expectFilesList()
    }

    // Text editor: edit, change, go back, error, retry, error, retry
    @Test
    fun testTextEditorEditChangeGoBackErrorRetryErrorRetry() {
        editFile()
        changeContent()

        simulateUploadError {
            goBack()
        }

        h.dialogWait(
            dialogTitle = "File could not be saved",
            dialogBodyRegex = "File could not be saved .*\\. Do you want to Try again or Discard the changes\\?",
            primaryButtonText = "Try again",
            secondaryButtonText = "Discard changes",
        )

        simulateUploadError {
            h.dialogButtonClick("Try again")
        }

        handleFileCouldNotBeSavedDialog("Try again")

        expectServerContentMatch()
        expectFilesList()
    }

    // Text editor: edit, change, go back, error, discard changes
    @Test
    fun testTextEditorEditChangeGoBackErrorDiscardChanges() {
        editFile()
        changeContent()

        simulateUploadError {
            goBack()
        }

        handleFileCouldNotBeSavedDialog("Discard changes")

        currentContent = "editorcontent"
        expectServerContentMatch()
        expectFilesList()
    }

    // Text editor: edit, change, autosave
    @Test
    fun testTextEditorEditChangeAutosave() {
        fixture.launchApp(intentExtra = mapOf("vaultTextEditorAutosaveIntervalMs" to "1000"))
        h.reposRepoClick()
        h.repoUnlock()

        editFile()
        changeContent()
        // content will be autosaved in 1s
        expectServerContentMatch()
    }

    // Text editor: edit, change, save, go back
    @Test
    fun testTextEditorEditChangeSaveGoBack() {
        editFile()
        changeContent()
        clickSave()
        goBack()
        expectServerContentMatch()
        expectFilesList()
    }

    // Text editor: edit, change, change on server
    @Test
    fun testTextEditorEditChangeSaveServerChange() {
        editFile()
        changeContent()
        changeContentOnServer()
        expectFileError("File was changed by someone else since your last save. Automatic saving is disabled.")
    }

    // Text editor: edit, change, change on server, save, cancel
    @Test
    fun testTextEditorEditChangeChangeOnServerSaveCancel() {
        editFile()
        changeContent()
        changeContentOnServer()
        clickSave()

        handleFileWasChangedDialog("Cancel", "Cancel")
        expectFileError("File was changed by someone else since your last save. Automatic saving is disabled.")
    }

    // Text editor: edit, change, change on server, save, save as new file
    @Test
    fun testTextEditorEditChangeChangeOnServerSaveSaveAsNewFile() {
        editFile()
        changeContent()
        changeContentOnServer()
        clickSave()

        handleFileWasChangedDialog("Save as a new file", "Cancel")

        reflectAutorename()
        expectServerContentMatch()
        expectHeaderNameMatch()
        expectNoConflicts()
        expectNotDirty()
    }

    // Text editor: edit, change, change on server, done, discard
    @Test
    fun testTextEditorEditChangeChangeOnServerDoneDiscard() {
        editFile()
        changeContent()
        changeContentOnServer()
        clickDone()

        handleFileWasChangedDialog("Discard changes", "Discard changes")
        expectViewerOpen()
        expectServerContentMatch()
    }

    // Text editor: edit, change, change on server, done, save as new file
    @Test
    fun testTextEditorEditChangeChangeOnServerDoneSaveAsNewFile() {
        editFile()
        changeContent()
        changeContentOnServer()
        clickDone()

        handleFileWasChangedDialog("Save as a new file", "Discard changes")
        reflectAutorename()
        expectViewerOpen()
        expectServerContentMatch()
        expectHeaderNameMatch()
    }

    // Text editor: edit, change, change on server, go back, discard
    @Test
    fun testTextEditorEditChangeChangeOnServerGoBackDiscard() {
        editFile()
        changeContent()
        changeContentOnServer()
        goBack()

        handleFileWasChangedDialog("Discard changes", "Discard changes")
        expectFilesList()
    }

    // Text editor: edit, change, change on server, go back, save as new file
    @Test
    fun testTextEditorEditChangeChangeOnServerGoBackSaveAsNewFile() {
        editFile()
        changeContent()
        changeContentOnServer()
        goBack()

        handleFileWasChangedDialog("Save as a new file", "Discard changes")
        reflectAutorename()
        expectFilesList()
        expectFilesListFile()
        expectServerContentMatch()
    }

    // Text editor: view, change on server, reloaded
    @Test
    fun testTextEditorViewChangeOnServerReloaded() {
        viewFile()
        changeContentOnServer()
        expectCurrentContent()
    }

    // Text editor: edit, change on server, reloaded
    @Test
    fun testTextEditorEditChangeOnServerReloaded() {
        editFile()
        changeContentOnServer()
        expectCurrentContent()
        expectNoConflicts()
        expectNotDirty()
    }

    // Text editor: view, rename on server
    @Test
    fun testTextEditorViewRenameOnServer() {
        viewFile()
        renameFileOnServer()
        expectHeaderNameMatch()
        expectCurrentContent()
    }

    // Text editor: edit, rename on server
    @Test
    fun testTextEditorEditRenameOnServer() {
        editFile()
        renameFileOnServer()
        expectHeaderNameMatch()
        expectCurrentContent()
        expectNoConflicts()
        expectNotDirty()
    }

    // Text editor: edit, change, rename on server
    @Test
    fun testTextEditorEditChangeRenameOnServer() {
        editFile()
        changeContent()
        renameFileOnServer()
        expectHeaderNameMatch()
        expectDirty()
    }

    // Text editor: edit, change, rename on server, save
    @Test
    fun testTextEditorEditChangeRenameOnServerSave() {
        editFile()
        changeContent()
        renameFileOnServer()
        expectHeaderNameMatch()
        expectDirty()
        clickSave()
        expectNotDirty()
        expectServerContentMatch()
    }

    // Text editor: view, delete, cancel
    @Test
    fun testTextEditorViewDeleteCancel() {
        viewFile()
        clickMoreDelete()

        handleDeleteFilesDialog("Cancel")

        expectViewerOpen()
        expectCurrentContent()
    }

    // Text editor: view, delete, confirm
    @Test
    fun testTextEditorViewDeleteConfirm() {
        viewFile()
        clickMoreDelete()

        handleDeleteFilesDialog("Delete")

        expectFilesList()
        expectFilesListFileNotExist()
    }

    // Text editor: edit, change, delete on server
    @Test
    fun testTextEditorEditChangeDeleteOnServer() {
        editFile()
        changeContent()
        deleteFileOnServer()

        handleFileNotAccessibleDialog()

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")
    }

    // Text editor: edit, change, delete on server, save, cancel
    @Test
    fun testTextEditorEditChangeDeleteOnServerSaveCancel() {
        editFile()
        changeContent()
        deleteFileOnServer()

        handleFileNotAccessibleDialog()

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")

        clickSave()

        handleFileNotAccessibleSaveDialog("Cancel", "Cancel")

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")
    }

    // Text editor: edit, change, delete on server, save, save to new location
    @Test
    fun testTextEditorEditChangeDeleteOnServerSaveSaveToNewLocation() {
        editFile()
        changeContent()
        deleteFileOnServer()

        handleFileNotAccessibleDialog()

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")

        clickSave()

        handleFileNotAccessibleSaveDialog("Save to a new location", "Cancel")

        handleFileLocationChangedDialog()

        expectFilesListFile()
    }

    // Text editor: edit, change, delete on server, go back, discard changes
    @Test
    fun testTextEditorEditChangeDeleteOnServerGoBackDiscardChanges() {
        editFile()
        changeContent()
        deleteFileOnServer()

        handleFileNotAccessibleDialog()

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")

        goBack()

        handleFileNotAccessibleSaveDialog("Discard changes", "Discard changes")

        expectFilesList()
        expectFilesListFileNotExist()
    }

    // Text editor: edit, change, delete on server, go back, save to new location
    @Test
    fun testTextEditorEditChangeDeleteOnServerGoBackSaveToNewLocation() {
        editFile()
        changeContent()
        deleteFileOnServer()

        handleFileNotAccessibleDialog()

        expectFileError("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")

        goBack()

        handleFileNotAccessibleSaveDialog("Save to a new location", "Discard changes")

        handleFileLocationChangedDialog()

        expectFilesListFile()
    }

    @Test
    fun testBackTransferAborted() {
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.jpg", "text")

        fixture.debugClient.downloadsPause()

        h.repoFilesFileRowClick("file.jpg")

        h.transfersButtonWaitVisible()

        goBack()

        h.transfersButtonWaitHidden()
    }
}
