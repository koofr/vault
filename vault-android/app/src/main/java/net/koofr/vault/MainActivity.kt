package net.koofr.vault

import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.appcompat.app.AppCompatActivity
import dagger.hilt.android.AndroidEntryPoint
import net.koofr.vault.features.auth.AuthGuard
import net.koofr.vault.features.intl.IntlHelper
import net.koofr.vault.features.mobilevault.MobileVaultProvider
import net.koofr.vault.features.mobilevault.isMobileVaultSupported
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {
    @Inject
    lateinit var mobileVaultProvider: MobileVaultProvider

    @Inject
    lateinit var intlHelper: IntlHelper

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        mobileVaultProvider.loadConfigFromIntent(intent)

        intlHelper.updateMobileVaultIntlCurrentLocale(resources.configuration)

        setContent {
            if (isMobileVaultSupported()) {
                CommonContent {
                    AuthGuard()
                }
            } else {
                NotSupported()
            }
        }
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)

        intlHelper.updateMobileVaultIntlCurrentLocale(newConfig)
    }
}
