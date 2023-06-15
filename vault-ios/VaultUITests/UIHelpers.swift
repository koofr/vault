import XCTest

extension XCUIApplication {
    // landing

    func landingGetStartedTap() {
        let button = buttons["Get started"]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    // auth

    func authContinueTap() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let button = springboard.buttons["Continue"]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    // repos

    func reposRepoTap(repoName: String = "My safe box") {
        let repo = collectionViews.buttons[repoName]
        XCTAssertTrue(repo.waitForExistence(timeout: 10))
        repo.tap()
    }

    func reposRepoInfoTap(repoName: String = "My safe box") {
        let repo = collectionViews.buttons[repoName]
        XCTAssertTrue(repo.waitForExistence(timeout: 10))

        collectionViews.buttons["Info"].tap()
    }

    func reposCreateNewTap() {
        collectionViews.buttons["Create new"].tap()
    }

    // repo unlock

    func repoUnlockWait(message: String = "Enter your Safe Key to continue") {
        XCTAssertTrue(staticTexts[message].waitForExistence(timeout: 10))
    }

    func repoUnlock(password: String = "password") {
        let field = secureTextFields["Safe Key"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        field.typeText(password)

        buttons["Continue"].tap()
    }

    // repo info

    func repoInfoBiometricUnlockTap() {
        let button = switches["Biometric unlock, Use biometrics to unlock the Safe Box"].switches
            .firstMatch
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    func repoInfoUnlockedTap() -> XCUIElement {
        let button = switches["Unlocked, Unlock or lock the Safe Box"].switches.firstMatch
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()

        return button
    }

    // repo create

    func repoCreateWait() {
        XCTAssertTrue(navigationBars["Create a new Safe Box"].waitForExistence(timeout: 10))
    }

    func repoCreateLocationTap() {
        let button = scrollViews["Location"]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    func repoCreatePasswordFill(password: String = "password") {
        let field = secureTextFields["Safe Key"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        field.tap()
        field.typeText(password)
    }

    func repoCreateAdvancedSettingsTap() {
        buttons["Show advanced settings"].tap()
    }

    func repoCreateSaltFill(salt: String = "salt") {
        let textView = textViews["Salt"]
        XCTAssertTrue(textView.waitForExistence(timeout: 10))

        // salt.tap() does not work. it fails to focus a textView (it works if
        // it's a textField). coordinate.tap() can be used to focus the view,
        // then tap() works
        let coordinate = textView.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
            .withOffset(CGVector(dx: 15, dy: 15))
        coordinate.tap()

        // empty value is not "" but placeholder Salt. loop because tripple-tap
        // might not select all text if the text contains "-" on the tap
        // coordinate
        while textView.value as? String? != "Salt" {
            // tripple-tap to select all text
            textView.tap(withNumberOfTaps: 3, numberOfTouches: 1)
            textView.typeText(XCUIKeyboardKey.delete.rawValue)
        }

        textView.typeText(salt)
    }

    func repoCreateCreateTap() {
        navigationBars["Create a new Safe Box"].buttons["Create"].tap()
    }

    func repoCreateCreatedSwipeUp() {
        scrollViews.staticTexts.element(boundBy: 0).swipeUp()
    }

    func repoCreateCreatedShareTap() {
        let button = buttons["Share…"]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    func repoCreateCreatedContinueTap() {
        navigationBars["Create a new Safe Box"].buttons["Continue"].tap()
    }

    // repo files

    func repoFilesFile(fileName: String) -> XCUIElement {
        return collectionViews.buttons.containing(.staticText, identifier: fileName).firstMatch
    }

    func repoFilesFileTap(fileName: String) {
        let file = repoFilesFile(fileName: fileName)
        XCTAssertTrue(file.waitForExistence(timeout: 10))
        file.tap()
    }

    func repoFilesFileWaitNotExist(fileName: String) {
        let file = repoFilesFile(fileName: fileName)
        XCTAssertTrue(
            waitFor {
                !file.exists
            })
    }

    func repoFilesBack(parentName: String) {
        navigationBars.buttons[parentName].tap()
    }

    func repoFilesMenuTap(repoName: String = "My safe box") {
        let button = navigationBars[repoName].images["More"]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    func repoFilesMenuSelectTap() {
        menuItemTap(itemName: "Select")
    }

    func repoFilesMenuNewFolderTap() {
        menuItemTap(itemName: "New folder")
    }

    func repoFilesFileContextMenu(fileName: String) {
        let file = repoFilesFile(fileName: fileName)
        XCTAssertTrue(file.waitForExistence(timeout: 10))
        file.press(forDuration: 1)
    }

    func repoFilesFileMenuMoveTap() {
        menuItemTap(itemName: "Move")
    }

    func repoFilesEditModeWait() {
        XCTAssertTrue(navigationBars["Selected items"].waitForExistence(timeout: 10))
    }

    func repoFilesEditModeWaitDisabled() {
        XCTAssertTrue(
            waitFor {
                !navigationBars["Selected items"].exists
            })
    }

    func repoFilesEditModeToolbarDeleteTap() {
        toolbars["Toolbar"].buttons["Delete selected"].tap()
    }

    // repo files move

    func repoFilesMoveNavigationBar() -> XCUIElement {
        return navigationBars.containing(.button, identifier: "Cancel").firstMatch
    }

    func repoFilesMoveWait() {
        XCTAssertTrue(repoFilesMoveNavigationBar().waitForExistence(timeout: 10))
    }

    func repoFilesMoveNavigationWait(folderName: String) {
        let navigationBar = navigationBars[folderName]
        XCTAssertTrue(navigationBar.waitForExistence(timeout: 10))
    }

    func repoFilesMoveMenuTap() {
        repoFilesMoveNavigationBar().images["More"].tap()
    }

    func repoFilesMoveMoveTap(buttonText: String = "Move") {
        let button = navigationBars.buttons[buttonText]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        XCTAssertTrue(
            waitFor {
                button.isEnabled
            })
        button.tap()
    }

    // transfers

    func transfersButton() -> XCUIElement {
        return navigationBars.buttons["Show transfers"]
    }

    func transfersButtonWait() {
        let button = transfersButton()
        XCTAssertTrue(button.waitForExistence(timeout: 10))
    }

    func transfersButtonWaitNotExist() {
        let button = transfersButton()
        XCTAssertTrue(
            waitFor {
                !button.exists
            })
    }

    func transfersButtonTap() {
        let button = transfersButton()
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    // dialogs

    func dialogPromptSubmit(dialogTitle: String, inputValue: String, submitButtonText: String) {
        let alert = alerts[dialogTitle]
        XCTAssertTrue(alert.waitForExistence(timeout: 10))
        alert.textFields.firstMatch.typeText(inputValue)
        alert.buttons[submitButtonText].tap()
    }

    func dialogsNewFolderSubmit(folderName: String) {
        dialogPromptSubmit(
            dialogTitle: "Enter new folder name", inputValue: folderName,
            submitButtonText: "Create folder")
    }

    func dialogConfirmSubmit(dialogTitle: String, submitButtonText: String) {
        let alert = alerts[dialogTitle]
        XCTAssertTrue(alert.waitForExistence(timeout: 10))
        alert.buttons[submitButtonText].tap()
    }

    func dialogsDeleteFilesSubmit() {
        dialogConfirmSubmit(
            dialogTitle: "Delete files",
            submitButtonText: "Delete")
    }

    // menus

    func menuItemTap(itemName: String) {
        let button = collectionViews.buttons[itemName]
        XCTAssertTrue(button.waitForExistence(timeout: 10))
        button.tap()
    }

    // wait

    // use when NSPredicate(format: "predicate") or waitForExpectations do not
    // work
    func waitFor(_ check: () -> Bool, intervalSeconds: Double = 0.01, timeoutSeconds: Double = 5)
        -> Bool
    {
        var waitedSeconds: Double = 0

        while waitedSeconds < timeoutSeconds {
            if check() {
                return true
            }

            usleep(UInt32(intervalSeconds / 1_000_000))

            waitedSeconds += intervalSeconds
        }

        return false
    }

    // popovers

    func dismissPopover() {
        // otherElements["PopoverDismissRegion"].tap() does not work
        navigationBars.element(boundBy: 0).coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
            .withOffset(CGVector(dx: 5, dy: 5)).tap()
    }
}
