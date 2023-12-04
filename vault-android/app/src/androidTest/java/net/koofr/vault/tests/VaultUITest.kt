package net.koofr.vault.tests

import android.os.SystemClock
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import org.junit.After
import org.junit.Assert
import org.junit.Ignore
import org.junit.Test
import org.junit.runner.RunWith

// tested on Pixel 4 API 30
@RunWith(AndroidJUnit4::class)
class VaultUITest {
    private lateinit var fixture: Fixture

    private fun build(authenticate: Boolean = true, createRepo: Boolean = true) {
        fixture = Fixture.build(authenticate = authenticate, createRepo = createRepo)
    }

    @After
    fun cleanup() {
        fixture.close()
    }

    @Test
    fun testLoginCreateRevokeLoginCreate() {
        build(authenticate = false, createRepo = false)
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.landingGetStartedClick()

        h.repoCreateWait()

        fixture.debugClient.oauth2Revoke()

        h.repoCreateLocationClick()

        h.landingGetStartedClick()

        h.repoCreateWait()
    }

    // this test cannot be automated and you need to touch the fingerprint
    // reader
    @Ignore
    @Test
    fun testRepoInfoSetupBiometricsAndUnlock() {
        build()
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoInfoClick()

        h.repoInfoBiometricUnlockClick()

        h.repoUnlock()

        h.fingerprintSheetWaitVisible()
        // touch the fingerprint, cannot automate this
        h.fingerprintSheetWaitHidden()

        h.repoInfoBiometricUnlockCheckedWait()

        h.repoInfoLockedClick()

        h.fingerprintSheetWaitVisible()
        // touch the fingerprint, cannot automate this
        h.fingerprintSheetWaitHidden()

        h.repoInfoUnlockedWait()
    }

    @Test
    fun testRepoInfoAutoLockAfter() {
        build()
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoInfoClick()

        h.reposRepoInfoLockAfterClick()

        device.findObject(By.text("10 minutes of inactivity")).click()

        h.dialogButtonClick("OK")

        device.pressBack()

        h.reposRepoInfoClick()

        Assert.assertEquals("10 minutes of inactivity", h.reposRepoInfoLockAfterValue())
    }

    @Test
    fun testRepoInfoAutoLockOnAppHidden() {
        build()
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoInfoClick()

        h.reposRepoInfoLockOnAppHiddenClick()

        device.pressBack()

        h.reposRepoInfoClick()

        Assert.assertTrue(h.reposRepoInfoLockAfterChecked())
    }

    @Test
    fun testRepoCreate() {
        build(authenticate = true, createRepo = false)
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.repoCreateWait()

        h.repoCreatePasswordFill()
        h.repoCreateAdvancedSettingsClick()
        h.repoCreateSaltFill()

        h.repoCreateCreateClick()

        h.repoCreateCreatedWait()
        h.repoCreateCreatedScrollDown()

        h.repoCreateCreatedShareClick()

        h.shareSheetWait()
        device.pressBack()

        h.repoCreateCreatedContinueClick()

        h.repoUnlockWait()
    }

    @Test
    fun testRepoFilesMoveToNewFolder() {
        build()
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesAddClick()
        h.repoFilesAddNewFolderClick()
        h.dialogsNewFolderSubmit("Foo")

        h.repoFilesFileRowMenuClick("Foo")
        h.repoFilesFileMenuMoveClick()

        h.repoFilesMoveWaitVisible()
        h.repoFilesMoveNewFolderClick()
        h.dialogsNewFolderSubmit("Bar")
        h.repoFilesMoveNavigationWait("Bar")
        h.repoFilesMoveMoveClick()
        h.repoFilesMoveWaitHidden()
    }

    @Test
    fun testRepoFilesSelectModeDelete() {
        build()
        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesAddClick()
        h.repoFilesAddNewFolderClick()
        h.dialogsNewFolderSubmit("Foo")

        h.repoFilesFileRowLongClick("Foo")

        h.repoFilesSelectModeWaitVisible()
        h.repoFilesDeleteSelectedClick()
        h.dialogsDeleteFilesSubmit()
        h.repoFilesFileRowWaitNotExist("Foo")
        h.repoFilesSelectModeWaitHidden()
    }

    @Test
    fun testRepoFilesAutoLockAfter() {
        build()
        val device = fixture.launchApp(
            mapOf(
                "vaultReposSetDefaultAutoLock" to "3",
            ),
        )
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesEmptyFolderWait()

        for (i in 1..5) {
            device.findObject(h.repoFilesEmptyFolderSelector).click()

            SystemClock.sleep(1000)
        }

        SystemClock.sleep(5000)

        h.repoUnlockWait()
    }

    @Test
    fun testRepoFilesAutoLockOnAppHidden() {
        build()
        val device = fixture.launchApp(
            mapOf(
                "vaultReposSetDefaultAutoLock" to "onapphidden",
            ),
        )
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesEmptyFolderWait()

        device.pressHome()

        fixture.activateApp()

        h.repoUnlockWait()
    }

    @Test
    fun testRepoFilesKeepSelectionOnLock() {
        build()

        val repo = fixture.mobileVaultHelper.waitForRepoUnlock()
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.txt", "čšž")

        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesFileRowLongClick("file.txt")

        h.repoFilesSelectModeWaitVisible()

        device.findObject(By.text("1 selected")).click(5000)

        h.repoUnlock()

        h.repoFilesSelectModeWaitVisible()
    }

    @Test
    fun testRepoFilesDetailsTextUtf8() {
        build()

        val repo = fixture.mobileVaultHelper.waitForRepoUnlock()
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.txt", "čšž")

        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesFileRowClick("file.txt")

        h.repoFilesDetailsContentTextWait("čšž")
    }

    @Test
    fun testRepoFilesDetailsBackTransferAborted() {
        build()

        val repo = fixture.mobileVaultHelper.waitForRepoUnlock()
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.jpg", "text")

        val device = fixture.launchApp()
        val h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        fixture.debugClient.downloadsPause()

        h.repoFilesFileRowClick("file.jpg")

        h.transfersButtonWaitVisible()

        device.pressBack()

        h.transfersButtonWaitHidden()
    }
}
