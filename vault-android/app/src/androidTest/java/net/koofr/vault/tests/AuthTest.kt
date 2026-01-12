package net.koofr.vault.tests

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import net.koofr.vault.tests.helpers.Fixture
import net.koofr.vault.tests.helpers.UIHelpers
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

// tested on Medium Phone API 36.1
@RunWith(AndroidJUnit4::class)
class AuthTest {
    private lateinit var fixture: Fixture
    private lateinit var device: UiDevice
    private lateinit var h: UIHelpers

    @Before
    fun setUp() {
        fixture = Fixture.build(authenticate = false, createRepo = false)
        device = fixture.launchApp()
        h = UIHelpers(device)
    }

    @After
    fun tearDown() {
        fixture.close()
    }

    @Test
    fun testLoginCreateRevokeLoginCreate() {
        h.landingGetStartedClick()

        h.repoCreateWait()

        fixture.debugClient.oauth2Revoke()

        h.repoCreateLocationClick()

        h.landingGetStartedClick()

        h.repoCreateWait()
    }
}
