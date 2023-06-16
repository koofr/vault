package net.koofr.vault.tests

import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.koofr.vault.FakeRemote
import net.koofr.vault.MobileVault
import org.json.JSONObject
import java.io.Closeable

class Fixture constructor(
    val fakeRemote: FakeRemote,
    val baseURL: String,
    val oauth2AuthBaseURL: String,
    val debugClient: DebugClient,
    val mobileVault: MobileVault,
    val mobileVaultHelper: MobileVaultHelper,
    val secureStorageJSON: String,
) : Closeable {
    private var activity: Activity? = null

    companion object {
        private const val LAUNCH_TIMEOUT = 5000L

        fun build(authenticate: Boolean = true, createRepo: Boolean = true): Fixture {
            val fakeRemote =
                FakeRemote(httpAddr = "127.0.0.1:0", httpsAddr = "127.0.0.1:0")

            val started = fakeRemote.start()

            val httpURL = started.httpUrl
            val httpsURL = started.httpsUrl

//            val httpURL = "https://10.0.2.2:3080"
//            val httpsURL = "https://10.0.2.2:3443"

            println("FakeRemote HTTP URL: $httpURL")
            println("FakeRemote HTTPS URL: $httpsURL")

            val baseURL = httpsURL
            val appName = "vault-android-test"
            val oauth2AuthBaseURL = httpURL
            val oauth2ClientId = "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3"
            val oauth2ClientSecret =
                "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB"
            val oauth2RedirectUri = "koofrvault://oauth2callback"
            val debugBaseURL = httpsURL

            val debugClient = DebugClient(debugBaseURL)

            debugClient.reset()

            if (createRepo) {
                debugClient.createTestVaultRepo()
            }

            val memorySecureStorage = MemorySecureStorage()

            if (authenticate) {
                memorySecureStorage.setItem(
                    "vaultOAuth2Token",
                    "{\"access_token\":\"\",\"refresh_token\":\"a126768a-ce0b-4b93-8a9b-809f02f4c000\",\"expires_at\":0}",
                )
            }

            val mobileVault =
                MobileVault(
                    baseURL,
                    appName,
                    oauth2AuthBaseURL,
                    oauth2ClientId,
                    oauth2ClientSecret,
                    oauth2RedirectUri,
                    memorySecureStorage,
                )
            val mobileVaultHelper = MobileVaultHelper(mobileVault)

            mobileVault.load()

            if (authenticate) {
                mobileVaultHelper.waitForOAuth2Loaded()
                mobileVaultHelper.waitForReposLoaded()
            }

            val secureStorageJSON =
                JSONObject(memorySecureStorage.getData() as Map<*, *>).toString()

            return Fixture(
                fakeRemote = fakeRemote,
                baseURL = baseURL,
                oauth2AuthBaseURL = oauth2AuthBaseURL,
                debugClient = debugClient,
                mobileVault = mobileVault,
                mobileVaultHelper = mobileVaultHelper,
                secureStorageJSON = secureStorageJSON,
            )
        }
    }

    override fun close() {
        try {
            stopApp()
        } catch (e: Exception) {
            println("Failed to kill the app: $e")
        }

        try {
            fakeRemote.stop()
        } catch (e: Exception) {
            println("Failed to stop fake remote: $e")
        }

        try {
            fakeRemote.destroy()
        } catch (e: Exception) {
            println("Failed to destroy fake remote: $e")
        }
    }

    fun launchApp(): UiDevice {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val context = instrumentation.context
        val targetContext = instrumentation.targetContext

        val device = UiDevice.getInstance(instrumentation)

        device.pressHome()

        val launcherPackage = getLauncherPackageName()

        device.wait(Until.hasObject(By.pkg(launcherPackage).depth(0)), LAUNCH_TIMEOUT)

        val packageManager = context.packageManager

        val appPackageName = targetContext.packageName

        val intent = checkNotNull(packageManager.getLaunchIntentForPackage(appPackageName)) {
            "Failed to create launch intent"
        }
        intent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
        intent.putExtra("vaultBaseUrl", baseURL)
        intent.putExtra("vaultOAuth2AuthBaseUrl", oauth2AuthBaseURL)
        intent.putExtra("vaultSecureStorage", secureStorageJSON)

        activity = instrumentation.startActivitySync(intent)

        device.wait(
            Until.hasObject(
                By.pkg(appPackageName)
                    .depth(0),
            ),
            LAUNCH_TIMEOUT,
        )

        return device
    }

    private fun stopApp() {
        activity?.finishAffinity()
    }
}

private fun getLauncherPackageName(): String {
    val intent = Intent(Intent.ACTION_MAIN)
    intent.addCategory(Intent.CATEGORY_HOME)

    val pm = InstrumentationRegistry.getInstrumentation().context.packageManager
    val resolveInfo = pm.resolveActivity(intent, PackageManager.MATCH_DEFAULT_ONLY)

    return checkNotNull(resolveInfo?.activityInfo?.packageName) {
        "Launcher package name not found"
    }
}
