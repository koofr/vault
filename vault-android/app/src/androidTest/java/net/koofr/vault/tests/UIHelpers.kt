package net.koofr.vault.tests

import android.widget.EditText
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import java.util.regex.Pattern

// use `adb exec-out uiautomator dump /dev/tty | code -` (and Format document)
// to see the view hierarchy

@Suppress("MemberVisibilityCanBePrivate")
class UIHelpers(private val device: UiDevice) {
    // landing

    val landingGetStartedSelector = By.clickable(true).hasDescendant(By.text("Get started"))

    fun landingGetStartedClick() {
        checkNotNull(
            device.wait(Until.findObject(landingGetStartedSelector), 10000),
        ) { "Get Started button was not found" }.click()
    }

    // repos

    fun reposRepoSelector(repoName: String) = By.clickable(true).hasDescendant(By.text(repoName))

    fun reposRepoClick(repoName: String = "My safe box") {
        checkNotNull(
            device.wait(Until.findObject(reposRepoSelector(repoName)), 10000),
        ) { "$repoName repo was not found" }.click()
    }

    fun reposRepoInfoSelector(repoName: String) =
        By.clickable(true).hasDescendant(By.desc("Info")).hasParent(By.hasChild(By.desc("Safe Box $repoName")))

    fun reposRepoInfoClick(repoName: String = "My safe box") {
        checkNotNull(
            device.wait(Until.findObject(reposRepoInfoSelector(repoName)), 10000),
        ) { "Info button for $repoName repo was not found" }.click()
    }

    // repo unlock

    val repoUnlockTitleSelector = By.text("Enter your Safe Key to continue")

    fun repoUnlockWait() {
        checkNotNull(
            device.wait(Until.findObject(repoUnlockTitleSelector), 10000),
        ) { "Unlock title was not found" }
    }

    val repoUnlockPaswordSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("Safe Key"))

    val repoUnlockContinueSelector = By.clickable(true).hasDescendant(By.text("Continue"))

    fun repoUnlock(password: String = "password") {
        repoUnlockWait()

        val field = checkNotNull(
            device.wait(Until.findObject(repoUnlockPaswordSelector), 10000),
        ) { "Password field was not found" }
        field.text = password

        checkNotNull(
            device.wait(Until.findObject(repoUnlockContinueSelector), 10000),
        ) { "Continue button was not found" }.click()
    }

    // repo info

    val repoInfoBiometricUnlockSelector = By.checkable(true).desc("Biometric unlock")

    fun repoInfoBiometricUnlockClick() {
        checkNotNull(
            device.wait(Until.findObject(repoInfoBiometricUnlockSelector), 10000),
        ) { "Biometric unlock checkbox was not found" }.click()
    }

    fun repoInfoBiometricUnlockCheckedWait() {
        checkNotNull(
            device.wait(Until.findObject(repoInfoBiometricUnlockSelector.checked(true)), 10000),
        ) { "Checked biometric unlock checkbox was not found" }
    }

    val repoInfoUnlockedSelector = By.checkable(true).desc("Unlocked")

    fun repoInfoUnlockedWait() {
        checkNotNull(
            device.wait(Until.findObject(repoInfoUnlockedSelector.checked(true)), 10000),
        ) { "Checked unlocked checkbox was not found" }
    }

    val repoInfoLockedSelector = By.checkable(true).desc("Locked")

    fun repoInfoLockedClick() {
        checkNotNull(
            device.wait(Until.findObject(repoInfoLockedSelector), 10000),
        ) { "Locked checkbox was not found" }.click()
    }

    val reposRepoInfoLockAfterSelector = By.text("Automatically lock after")

    fun reposRepoInfoLockAfterClick() {
        checkNotNull(
            device.wait(Until.findObject(reposRepoInfoLockAfterSelector), 10000),
        ) { "Lock after selector was not found" }.click()
    }

    val reposRepoInfoLockAfterValueSelector = By.desc("Automatically lock after value")

    fun reposRepoInfoLockAfterValue(): String {
        val obj = checkNotNull(
            device.wait(Until.findObject(reposRepoInfoLockAfterValueSelector), 10000),
        ) { "Lock after value was not found" }
        return obj.text
    }

    val reposRepoInfoLockOnAppHiddenSelector = By.checkable(true).desc("Lock when app hidden")

    fun reposRepoInfoLockOnAppHiddenClick() {
        checkNotNull(
            device.wait(Until.findObject(reposRepoInfoLockOnAppHiddenSelector), 10000),
        ) { "Lock on app hidden checkbox was not found" }.click()
    }

    fun reposRepoInfoLockAfterChecked(): Boolean {
        val obj = checkNotNull(
            device.wait(Until.findObject(reposRepoInfoLockOnAppHiddenSelector), 10000),
        ) { "Lock on app hidden checkbox was not found" }
        return obj.isChecked
    }

    // repo create

    val repoCreateTitleSelector = By.text("Create a new Safe Box")

    fun repoCreateWait() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateTitleSelector), 10000),
        ) { "Create repo title was not found" }
    }

    val repoCreateLocationSelector = By.clickable(true).hasDescendant(By.desc("Location"))

    fun repoCreateLocationClick() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateLocationSelector), 10000),
        ) { "Location selector was not found" }.click()
    }

    val repoCreatePasswordSelector =
        By.clazz(EditText::class.java).hasDescendant(By.desc("Safe Key"))

    fun repoCreatePasswordFill(password: String = "password") {
        val field = checkNotNull(
            device.wait(Until.findObject(repoCreatePasswordSelector), 10000),
        ) { "Password field was not found" }
        field.text = password
    }

    val repoCreateAdvancedSettingsSelector =
        By.clickable(true).hasDescendant(By.text("Show advanced settings"))

    fun repoCreateAdvancedSettingsClick() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateAdvancedSettingsSelector), 10000),
        ) { "Advanced settings button was not found" }.click()
    }

    val repoCreateSaltSelector = By.clazz(EditText::class.java).hasDescendant(By.desc("Salt"))

    fun repoCreateSaltFill(salt: String = "salt") {
        val field = checkNotNull(
            device.wait(Until.findObject(repoCreateSaltSelector), 10000),
        ) { "Salt field was not found" }
        field.text = salt
    }

    val repoCreateCreateSelector = By.clickable(true).hasDescendant(By.text("Create"))

    fun repoCreateCreateClick() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateCreateSelector), 10000),
        ) { "Create button was not found" }.click()
    }

    val repoCreateCreatedSelector = By.textStartsWith("Your Safe Box has been created")

    fun repoCreateCreatedWait() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateCreatedSelector), 10000),
        ) { "Created message was not found" }
    }

    fun repoCreateCreatedScrollDown() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateCreatedSelector), 10000),
        ) { "Created message was not found" }.fling(Direction.DOWN, 10000)
    }

    val repoCreateCreatedShareSelector = By.clickable(true).hasDescendant(By.text("Share…"))

    fun repoCreateCreatedShareClick() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateCreatedShareSelector), 10000),
        ) { "Share button was not found" }.click()
    }

    val repoCreateCreatedContinueSelector = By.clickable(true).hasDescendant(By.text("Continue"))

    fun repoCreateCreatedContinueClick() {
        checkNotNull(
            device.wait(Until.findObject(repoCreateCreatedContinueSelector), 10000),
        ) { "Continue button was not found" }.click()
    }

    // repo files

    val repoFilesEmptyFolderSelector = By.text("Folder is empty")

    fun repoFilesEmptyFolderWait() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesEmptyFolderSelector), 10000),
        ) { "Empty folder text was not found" }
    }

    fun repoFilesFileRowSelector(fileName: String) =
        By.clickable(true).hasDescendant(
            By.text(fileName),
        )

    fun repoFilesFileRowWait(fileName: String) {
        checkNotNull(
            device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000),
        ) { "$fileName file row was not found" }
    }

    fun repoFilesFileRowWaitNotExist(fileName: String) {
        check(device.wait(Until.gone(repoFilesFileRowSelector(fileName)), 10000)) {
            "$fileName file row did not disappear"
        }
    }

    fun repoFilesFileRowClick(fileName: String) {
        checkNotNull(
            device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000),
        ) { "$fileName file row was not found" }.click()
    }

    fun repoFilesFileRowLongClick(fileName: String) {
        checkNotNull(
            device.wait(Until.findObject(repoFilesFileRowSelector(fileName)), 10000),
        ) { "$fileName file row was not found" }.longClick()
    }

    fun repoFilesFileRowMenuSelector(fileName: String) =
        By.clickable(true).hasDescendant(By.desc("File menu")).hasAncestor(
            By.clickable(true).hasDescendant(
                By.text(fileName),
            ),
        )

    fun repoFilesFileRowMenuClick(fileName: String) {
        checkNotNull(
            device.wait(Until.findObject(repoFilesFileRowMenuSelector(fileName)), 10000),
        ) { "$fileName file menu was not found" }.click()
    }

    fun repoFilesFileMenuMoveClick() {
        menuItemClick("Move")
    }

    val repoFilesMenuSelector = By.clickable(true).hasDescendant(By.desc("Menu"))

    fun repoFilesMenuClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesMenuSelector), 10000),
        ) { "Files menu was not found" }.click()
    }

    val repoFilesAddSelector = By.clickable(true).hasDescendant(By.desc("Add"))

    fun repoFilesAddClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesAddSelector), 10000),
        ) { "Add button was not found" }.click()
    }

    val repoFilesAddNewFolderSelector = By.clickable(true).hasDescendant(By.text("New folder"))

    fun repoFilesAddNewFolderClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesAddNewFolderSelector), 10000),
        ) { "New folder button was not found" }.click()
    }

    fun repoFilesSelectModeWaitVisible(text: String = "1 selected") {
        checkNotNull(
            device.wait(Until.findObject(By.text(text)), 10000),
        ) { "$text was not found" }
    }

    fun repoFilesSelectModeWaitHidden(text: String = "1 selected") {
        check(device.wait(Until.gone(By.text(text)), 10000)) {
            "$text did not disappear"
        }
    }

    val repoFilesDeleteSelectedSelector =
        By.clickable(true).hasDescendant(By.desc("Delete selected"))

    fun repoFilesDeleteSelectedClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesDeleteSelectedSelector), 10000),
        ) { "Delete selected button was not found" }.click()
    }

    // repo files details

    fun repoFilesDetailsTextEditorContentWait(text: String) {
        checkNotNull(
            device.wait(Until.findObject(By.text(text)), 10000),
        ) { "Text '$text' was not found in content" }
    }

    // repo files move

    val repoFilesMoveSelector = By.clickable(true).hasDescendant(By.text("CANCEL"))

    fun repoFilesMoveWaitVisible() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesMoveSelector), 10000),
        ) { "Move dialog was not found" }
    }

    fun repoFilesMoveWaitHidden() {
        check(device.wait(Until.gone(repoFilesMoveSelector), 10000)) {
            "Move dialog did not disappear"
        }
    }

    val repoFilesMoveNewFolderSelector = By.clickable(true).hasDescendant(By.desc("New folder"))

    fun repoFilesMoveNewFolderClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesMoveNewFolderSelector), 10000),
        ) { "New folder in move dialog was not found" }.click()
    }

    fun repoFilesMoveNavigationWait(folderName: String) {
        checkNotNull(
            device.wait(Until.findObject(By.text(folderName)), 10000),
        ) { "$folderName folder was not found in move dialog" }.click()
    }

    val repoFilesMoveMoveSelector = By.clickable(true).hasDescendant(By.text("MOVE"))

    fun repoFilesMoveMoveClick() {
        checkNotNull(
            device.wait(Until.findObject(repoFilesMoveMoveSelector), 10000),
        ) { "Move button was not found" }.click()
    }

    // transfers

    val transfersButtonSelector = By.clickable(true).hasDescendant(By.desc("Transfers"))

    fun transfersButtonWaitVisible() {
        checkNotNull(
            device.wait(Until.findObject(transfersButtonSelector), 10000),
        ) { "Transfers button was not found" }
    }

    fun transfersButtonWaitHidden() {
        check(device.wait(Until.gone(transfersButtonSelector), 10000)) {
            "Transfers button did not disappear"
        }
    }

    // dialogs

    fun dialogWaitVisible(dialogTitle: String) {
        checkNotNull(
            device.wait(Until.findObject(By.text(dialogTitle)), 10000),
        ) { "Dialog '$dialogTitle' was not found" }
    }

    fun dialogWaitHidden(dialogTitle: String) {
        check(device.wait(Until.gone(By.text(dialogTitle)), 10000)) {
            "Dialog '$dialogTitle' did not disappear"
        }
    }

    fun dialogButtonClick(buttonText: String) {
        checkNotNull(
            device.wait(
                Until.findObject(
                    By.clickable(true).enabled(true)
                        .hasDescendant(By.text(buttonText.uppercase())),
                ),
                10000,
            ),
        ) { "$buttonText button was not found" }.click()
    }

    fun dialogPromptSubmit(dialogTitle: String, inputValue: String, submitButtonText: String) {
        dialogWaitVisible(dialogTitle)

        val field = checkNotNull(
            device.wait(Until.findObject(By.clazz(EditText::class.java)), 10000),
        ) { "Input field was not found" }
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
        checkNotNull(
            device.wait(
                Until.findObject(
                    By.clickable(true).enabled(true)
                        .hasDescendant(By.text(itemName)),
                ),
                10000,
            ),
        ) { "$itemName menu item was not found" }.click()
    }

    // share sheet

    val shareSheetSelector = By.text(Pattern.compile("Share|Sharing text"))

    fun shareSheetWait() {
        checkNotNull(
            device.wait(Until.findObject(shareSheetSelector), 10000),
        ) { "Share sheet was not found" }
    }

    // fingerprint

    val fingerprintSheetSelector = By.text("Safe Key biometrics")

    fun fingerprintSheetWaitVisible() {
        checkNotNull(
            device.wait(Until.findObject(fingerprintSheetSelector), 10000),
        ) { "Fingerprint sheet was not found" }
    }

    fun fingerprintSheetWaitHidden() {
        check(device.wait(Until.gone(fingerprintSheetSelector), 10000)) {
            "Fingerprint sheet did not disappear"
        }
    }
}
