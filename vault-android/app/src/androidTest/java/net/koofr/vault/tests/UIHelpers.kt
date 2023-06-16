package net.koofr.vault.tests

import android.widget.EditText
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until

// use `adb exec-out uiautomator dump /dev/tty | code -` (and Format document)
// to see the view hierarchy

@Suppress("MemberVisibilityCanBePrivate")
class UIHelpers(private val device: UiDevice) {
    // landing

    val landingGetStartedSelector = By.clickable(true).hasDescendant(By.text("Get started"))

    fun landingGetStartedClick() {
        device.wait(Until.findObject(landingGetStartedSelector), 10000).click()
    }

    // repos

    fun reposRepoSelector(repoName: String) = By.clickable(true).hasDescendant(By.text(repoName))

    fun reposRepoClick(repoName: String = "My safe box") {
        device.wait(Until.findObject(reposRepoSelector(repoName)), 10000).click()
    }

    fun reposRepoInfoSelector(repoName: String) =
        By.clickable(true).hasDescendant(By.desc("Info")).hasParent(By.hasChild(By.desc("Safe Box $repoName")))

    fun reposRepoInfoClick(repoName: String = "My safe box") {
        device.wait(Until.findObject(reposRepoInfoSelector(repoName)), 10000).click()
    }

    // repo unlock

    val repoUnlockTitleSelector = By.text("Enter your Safe Key to continue")

    fun repoUnlockWait() {
        device.wait(Until.findObject(repoUnlockTitleSelector), 10000)
    }

    val repoUnlockPaswordSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("Safe Key"))

    val repoUnlockContinueSelector = By.clickable(true).hasDescendant(By.text("Continue"))

    fun repoUnlock(password: String = "password") {
        repoUnlockWait()

        val field = device.wait(Until.findObject(repoUnlockPaswordSelector), 10000)
        field.text = password

        device.wait(Until.findObject(repoUnlockContinueSelector), 10000).click()
    }

    // repo info

    val repoInfoBiometricUnlockSelector = By.checkable(true).desc("Biometric unlock")

    fun repoInfoBiometricUnlockClick() {
        device.wait(Until.findObject(repoInfoBiometricUnlockSelector), 10000).click()
    }

    fun repoInfoBiometricUnlockCheckedWait() {
        device.wait(Until.findObject(repoInfoBiometricUnlockSelector.checked(true)), 10000)
    }

    val repoInfoUnlockedSelector = By.checkable(true).desc("Unlocked")

    fun repoInfoUnlockedWait() {
        device.wait(Until.findObject(repoInfoUnlockedSelector.checked(true)), 10000)
    }

    val repoInfoLockedSelector = By.checkable(true).desc("Locked")

    fun repoInfoLockedClick() {
        device.wait(Until.findObject(repoInfoLockedSelector), 10000).click()
    }

    // repo create

    val repoCreateTitleSelector = By.text("Create a new Safe Box")

    fun repoCreateWait() {
        device.wait(Until.findObject(repoCreateTitleSelector), 10000)
    }

    val repoCreateLocationSelector = By.clickable(true).hasDescendant(By.desc("Location"))

    fun repoCreateLocationClick() {
        device.wait(Until.findObject(repoCreateLocationSelector), 10000).click()
    }

    val repoCreatePasswordSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("Safe Key"))

    fun repoCreatePasswordFill(password: String = "password") {
        val field = device.wait(Until.findObject(repoCreatePasswordSelector), 10000)
        field.text = password
    }

    val repoCreateAdvancedSettingsSelector =
        By.clickable(true).hasDescendant(By.text("Show advanced settings"))

    fun repoCreateAdvancedSettingsClick() {
        device.wait(Until.findObject(repoCreateAdvancedSettingsSelector), 10000).click()
    }

    val repoCreateSaltSelector = By.clazz(EditText::class.java).hasDescendant(By.desc("Salt"))

    fun repoCreateSaltFill(salt: String = "salt") {
        val field = device.wait(Until.findObject(repoCreateSaltSelector), 10000)
        field.text = salt
    }

    val repoCreateCreateSelector = By.clickable(true).hasDescendant(By.text("Create"))

    fun repoCreateCreateClick() {
        device.wait(Until.findObject(repoCreateCreateSelector), 10000).click()
    }

    val repoCreateCreatedSelector = By.textStartsWith("Your Safe Box has been created")

    fun repoCreateCreatedWait() {
        device.wait(Until.findObject(repoCreateCreatedSelector), 10000)
    }

    fun repoCreateCreatedScrollDown() {
        device.wait(Until.findObject(repoCreateCreatedSelector), 10000).fling(Direction.DOWN, 10000)
    }

    val repoCreateCreatedShareSelector = By.clickable(true).hasDescendant(By.text("Share…"))

    fun repoCreateCreatedShareClick() {
        device.wait(Until.findObject(repoCreateCreatedShareSelector), 10000).click()
    }

    val repoCreateCreatedContinueSelector = By.clickable(true).hasDescendant(By.text("Continue"))

    fun repoCreateCreatedContinueClick() {
        device.wait(Until.findObject(repoCreateCreatedContinueSelector), 10000).click()
    }

    // repo files

    fun repoFilesFileRowSelector(fileName: String) =
        By.clickable(true).hasDescendant(
            By.text(fileName),
        )

    fun repoFilesFileRowWait(fileName: String) {
        device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000)
    }

    fun repoFilesFileRowWaitNotExist(fileName: String) {
        device.wait(Until.gone(repoFilesFileRowSelector(fileName)), 10000)
    }

    fun repoFilesFileRowClick(fileName: String) {
        device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000).click()
    }

    fun repoFilesFileRowLongClick(fileName: String) {
        device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000).longClick()
    }

    fun repoFilesFileRowMenuSelector(fileName: String) =
        By.clickable(true).hasDescendant(By.desc("File menu")).hasAncestor(
            By.clickable(true).hasDescendant(
                By.text(fileName),
            ),
        )

    fun repoFilesFileRowMenuClick(fileName: String) {
        device.wait(Until.findObject(repoFilesFileRowMenuSelector(fileName)), 10000).click()
    }

    fun repoFilesFileMenuMoveClick() {
        menuItemClick("Move")
    }

    val repoFilesMenuSelector = By.clickable(true).hasDescendant(By.desc("Menu"))

    fun repoFilesMenuClick() {
        device.wait(Until.findObject(repoFilesMenuSelector), 10000).click()
    }

    val repoFilesAddSelector = By.clickable(true).hasDescendant(By.desc("Add"))

    fun repoFilesAddClick() {
        device.wait(Until.findObject(repoFilesAddSelector), 10000).click()
    }

    val repoFilesAddNewFolderSelector = By.clickable(true).hasDescendant(By.text("New folder"))

    fun repoFilesAddNewFolderClick() {
        device.wait(Until.findObject(repoFilesAddNewFolderSelector), 10000).click()
    }

    fun repoFilesSelectModeWaitVisible(text: String = "1 selected") {
        device.wait(Until.findObject(By.text(text)), 10000)
    }

    fun repoFilesSelectModeWaitHidden(text: String = "1 selected") {
        device.wait(Until.gone(By.text(text)), 10000)
    }

    val repoFilesDeleteSelectedSelector =
        By.clickable(true).hasDescendant(By.desc("Delete selected"))

    fun repoFilesDeleteSelectedClick() {
        device.wait(Until.findObject(repoFilesDeleteSelectedSelector), 10000).click()
    }

    // repo files details

    val repoFilesDetailsContentTextSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("File text"))

    fun repoFilesDetailsContentTextWait(text: String) {
        device.wait(Until.findObject(repoFilesDetailsContentTextSelector.text(text)), 10000)
    }

    // repo files move

    val repoFilesMoveSelector = By.clickable(true).hasDescendant(By.text("CANCEL"))

    fun repoFilesMoveWaitVisible() {
        device.wait(Until.findObject(repoFilesMoveSelector), 10000)
    }

    fun repoFilesMoveWaitHidden() {
        device.wait(Until.gone(repoFilesMoveSelector), 10000)
    }

    val repoFilesMoveNewFolderSelector = By.clickable(true).hasDescendant(By.desc("New folder"))

    fun repoFilesMoveNewFolderClick() {
        device.wait(Until.findObject(repoFilesMoveNewFolderSelector), 10000).click()
    }

    fun repoFilesMoveNavigationWait(folderName: String) {
        device.wait(Until.findObject(By.text(folderName)), 10000).click()
    }

    val repoFilesMoveMoveSelector = By.clickable(true).hasDescendant(By.text("MOVE"))

    fun repoFilesMoveMoveClick() {
        device.wait(Until.findObject(repoFilesMoveMoveSelector), 10000).click()
    }

    // transfers

    val transfersButtonSelector = By.clickable(true).hasDescendant(By.desc("Transfers"))

    fun transfersButtonWaitVisible() {
        device.wait(Until.findObject(transfersButtonSelector), 10000)
    }

    fun transfersButtonWaitHidden() {
        device.wait(Until.gone(transfersButtonSelector), 10000)
    }

    // dialogs

    fun dialogWaitVisible(dialogTitle: String) {
        device.wait(Until.findObject(By.text(dialogTitle)), 10000)
    }

    fun dialogWaitHidden(dialogTitle: String) {
        device.wait(Until.gone(By.text(dialogTitle)), 10000)
    }

    fun dialogButtonClick(buttonText: String) {
        device.wait(
            Until.findObject(
                By.clickable(true).enabled(true)
                    .hasDescendant(By.text(buttonText.uppercase())),
            ),
            10000,
        ).click()
    }

    fun dialogPromptSubmit(dialogTitle: String, inputValue: String, submitButtonText: String) {
        dialogWaitVisible(dialogTitle)

        val field = device.wait(Until.findObject(By.clazz(EditText::class.java)), 10000)
        field.text = inputValue

        dialogButtonClick(submitButtonText)

        dialogWaitHidden(dialogTitle)
    }

    fun dialogsNewFolderSubmit(folderName: String) {
        dialogPromptSubmit(
            "Enter new folder name",
            folderName,
            "Create folder",
        )
    }

    fun dialogConfirmSubmit(dialogTitle: String, submitButtonText: String) {
        dialogWaitVisible(dialogTitle)

        dialogButtonClick(submitButtonText)

        dialogWaitHidden(dialogTitle)
    }

    fun dialogsDeleteFilesSubmit() {
        dialogConfirmSubmit("Delete files", "Delete")
    }

    // menu

    fun menuItemClick(itemName: String) {
        device.wait(
            Until.findObject(
                By.clickable(true).enabled(true)
                    .hasDescendant(By.text(itemName)),
            ),
            10000,
        ).click()
    }

    // share sheet

    val shareSheetSelector = By.text("Share")

    fun shareSheetWait() {
        device.wait(Until.findObject(shareSheetSelector), 10000)
    }

    // fingerprint

    val fingerprintSheetSelector = By.text("Safe Key biometrics")

    fun fingerprintSheetWaitVisible() {
        device.wait(Until.findObject(fingerprintSheetSelector), 10000)
    }

    fun fingerprintSheetWaitHidden() {
        device.wait(Until.gone(fingerprintSheetSelector), 10000)
    }
}
