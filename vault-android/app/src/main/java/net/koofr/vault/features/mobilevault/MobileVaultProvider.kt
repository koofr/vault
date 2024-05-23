package net.koofr.vault.features.mobilevault

import android.content.Intent
import com.sun.jna.Library
import com.sun.jna.Native
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.BuildConfig
import net.koofr.vault.LoggerLevel
import net.koofr.vault.MobileVault
import net.koofr.vault.RepoAutoLock
import net.koofr.vault.RepoAutoLockAfter
import net.koofr.vault.SecureStorage
import net.koofr.vault.setLogger
import org.json.JSONObject
import javax.inject.Singleton

data class Config(
    val baseUrl: String,
    val oauth2AuthBaseUrl: String,
    val secureStorageJSON: String?,
    val reposSetDefaultAutoLock: String?,
)

class MobileVaultProvider constructor(private val secureStorage: SecureStorage) {
    private var config: Config? = null

    private var mobileVault: MobileVault? = null
    private var currentMobileVaultConfig: Config? = null

    private val defaultConfig = Config(
        baseUrl = "https://app.koofr.net",
        oauth2AuthBaseUrl = "https://app.koofr.net",
        secureStorageJSON = null,
        reposSetDefaultAutoLock = null,
    )

    @Synchronized
    fun getConfig(): Config {
        if (config == null) {
            config = defaultConfig
        }

        return config!!
    }

    @Synchronized
    fun getMobileVault(): MobileVault {
        val config = getConfig()

        if (mobileVault == null || config != currentMobileVaultConfig) {
            mobileVault?.let {
                it.close()
            }
            mobileVault = buildMobileVault(config)
            currentMobileVaultConfig = config
        }

        return mobileVault!!
    }

    @Synchronized
    fun loadConfigFromIntent(intent: Intent) {
        if (BuildConfig.BUILD_TYPE != "debug") {
            // only allowed for testing in debug builds
            return
        }

        config = Config(
            baseUrl = intent.getStringExtra("vaultBaseUrl") ?: defaultConfig.baseUrl,
            oauth2AuthBaseUrl = intent.getStringExtra("vaultOAuth2AuthBaseUrl")
                ?: defaultConfig.oauth2AuthBaseUrl,
            secureStorageJSON = intent.getStringExtra("vaultSecureStorage"),
            reposSetDefaultAutoLock = intent.getStringExtra("vaultReposSetDefaultAutoLock"),
        )
    }

    private fun buildMobileVault(config: Config): MobileVault {
        config.secureStorageJSON?.let {
            secureStorage.clear()

            val obj = JSONObject(it)

            for (key in obj.keys()) {
                secureStorage.setItem(key, obj.getString(key))
            }
        }

        val baseUrl = config.baseUrl
        val appName = "vault-android"
        val oauth2AuthBaseUrl = config.oauth2AuthBaseUrl
        val oauth2ClientId = "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3"
        val oauth2ClientSecret = "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB"
        val oauth2RedirectUri = "koofrvault://oauth2callback"

        val mobileVault =
            MobileVault(
                baseUrl,
                appName,
                oauth2AuthBaseUrl,
                oauth2ClientId,
                oauth2ClientSecret,
                oauth2RedirectUri,
                secureStorage,
            )

        config.reposSetDefaultAutoLock?.let { autoLock ->
            if (autoLock == "onapphidden") {
                mobileVault.reposSetDefaultAutoLock(autoLock = RepoAutoLock(after = RepoAutoLockAfter.NoLimit, onAppHidden = true))
            } else {
                autoLock.toIntOrNull()?.let { seconds ->
                    mobileVault.reposSetDefaultAutoLock(autoLock = RepoAutoLock(after = RepoAutoLockAfter.Custom(seconds = seconds.toULong()), onAppHidden = false))
                }
            }
        }

        mobileVault.load()

        return mobileVault
    }
}

@Module
@InstallIn(SingletonComponent::class)
object MobileVaultProviderModule {
    @Singleton
    @Provides
    fun provideMobileVaultProvider(secureStorage: SecureStorage): MobileVaultProvider {
        setLogger(LoggerLevel.DEBUG, AndroidLogger())

        return MobileVaultProvider(secureStorage)
    }
}

fun isMobileVaultSupported(): Boolean {
    return try {
        Native.load("vault_mobile", Library::class.java)

        true
    } catch (t: Throwable) {
        false
    }
}
