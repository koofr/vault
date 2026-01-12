package net.koofr.vault.tests

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import net.koofr.vault.Repo
import net.koofr.vault.tests.helpers.Fixture
import net.koofr.vault.tests.helpers.UIHelpers
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

    @Before
    fun setUp() {
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

    @Test
    fun testTextEditorViewUtf8() {
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.txt", "čšž")

        h.repoFilesFileRowClick("file.txt")

        h.repoFilesDetailsTextEditorContentWait("čšž")
    }

    @Test
    fun testBackTransferAborted() {
        fixture.mobileVaultHelper.uploadFile(repo, "/", "file.jpg", "text")

        fixture.debugClient.downloadsPause()

        h.repoFilesFileRowClick("file.jpg")

        h.transfersButtonWaitVisible()

        device.pressBack()

        h.transfersButtonWaitHidden()
    }
}
