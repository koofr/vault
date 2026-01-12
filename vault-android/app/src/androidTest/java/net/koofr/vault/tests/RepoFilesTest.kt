package net.koofr.vault.tests

import android.os.SystemClock
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import net.koofr.vault.tests.helpers.Fixture
import net.koofr.vault.tests.helpers.UIHelpers
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

// tested on Medium Phone API 36.1
@RunWith(AndroidJUnit4::class)
class RepoFilesTest {
    private lateinit var fixture: Fixture
    private lateinit var device: UiDevice
    private lateinit var h: UIHelpers

    @Before
    fun setUp() {
        fixture = Fixture.build(authenticate = true, createRepo = true)
        device = fixture.launchApp()
        h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()
    }

    @After
    fun tearDown() {
        fixture.close()
    }

    @Test
    fun testMoveToNewFolder() {
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
    fun testSelectModeDelete() {
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
    fun testAutoLockAfter() {
        device = fixture.launchApp(
            mapOf(
                "vaultReposSetDefaultAutoLock" to "3",
            ),
        )
        h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesEmptyFolderWait()

        (1..5).forEach { i ->
            device.findObject(h.repoFilesEmptyFolderSelector).click()

            SystemClock.sleep(1000)
        }

        SystemClock.sleep(5000)

        h.repoUnlockWait()
    }

    @Test
    fun testAutoLockOnAppHidden() {
        device = fixture.launchApp(
            mapOf(
                "vaultReposSetDefaultAutoLock" to "onapphidden",
            ),
        )
        h = UIHelpers(device)

        h.reposRepoClick()
        h.repoUnlock()

        h.repoFilesEmptyFolderWait()

        device.pressHome()

        fixture.activateApp()

        h.repoUnlockWait()
    }

    @Test
    fun testKeepSelectionOnLock() {
        val repo = fixture.mobileVaultHelper.waitForRepoUnlock()
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.txt", "čšž")

        h.repoFilesFileRowLongClick("file.txt")

        h.repoFilesSelectModeWaitVisible()

        device.findObject(By.text("1 selected")).click(5000)

        h.repoUnlock()

        h.repoFilesSelectModeWaitVisible()
    }
}
