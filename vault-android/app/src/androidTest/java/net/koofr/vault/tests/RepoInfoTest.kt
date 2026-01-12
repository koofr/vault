package net.koofr.vault.tests

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import net.koofr.vault.tests.helpers.Fixture
import net.koofr.vault.tests.helpers.UIHelpers
import org.junit.After
import org.junit.Before
import org.junit.Ignore
import org.junit.Test
import org.junit.runner.RunWith

// tested on Medium Phone API 36.1
@RunWith(AndroidJUnit4::class)
class RepoInfoTest {
    private lateinit var fixture: Fixture
    private lateinit var device: UiDevice
    private lateinit var h: UIHelpers

    @Before
    fun setUp() {
        fixture = Fixture.build(authenticate = true, createRepo = true)
        device = fixture.launchApp()
        h = UIHelpers(device)
    }

    @After
    fun tearDown() {
        fixture.close()
    }

    // this test cannot be automated and you need to touch the fingerprint
    // reader
    @Ignore
    @Test
    fun testSetupBiometricsAndUnlock() {
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
    fun testAutoLockAfter() {
        h.reposRepoInfoClick()

        h.reposRepoInfoLockAfterClick()

        device.findObject(By.text("10 minutes of inactivity")).click()

        h.dialogButtonClick("OK")

        device.pressBack()

        h.reposRepoInfoClick()

        h.reposRepoInfoLockAfterValue().let { actual ->
            check(actual == "10 minutes of inactivity") {
                "Expected '10 minutes of inactivity' but was '$actual'"
            }
        }
    }

    @Test
    fun testAutoLockOnAppHidden() {
        h.reposRepoInfoClick()

        h.reposRepoInfoLockOnAppHiddenClick()

        device.pressBack()

        h.reposRepoInfoClick()

        check(h.reposRepoInfoLockAfterChecked()) {
            "Expected reposRepoInfoLockAfterChecked to be true"
        }
    }
}
