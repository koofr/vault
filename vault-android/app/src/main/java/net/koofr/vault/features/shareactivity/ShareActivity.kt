package net.koofr.vault.features.shareactivity

import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import dagger.hilt.android.AndroidEntryPoint
import net.koofr.vault.CommonContent
import net.koofr.vault.features.intl.IntlHelper
import net.koofr.vault.features.mobilevault.MobileVaultProvider
import javax.inject.Inject

@AndroidEntryPoint
class ShareActivity : AppCompatActivity() {
    @Inject
    lateinit var mobileVaultProvider: MobileVaultProvider

    @Inject
    lateinit var intlHelper: IntlHelper

    private val vm: ShareActivityViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        mobileVaultProvider.loadConfigFromIntent(intent)

        intlHelper.updateMobileVaultIntlCurrentLocale(resources.configuration)

        vm.onCancel = {
            setResult(RESULT_CANCELED)
            finish()
        }

        vm.onDone = {
            setResult(RESULT_OK)
            finish()
        }

        vm.initFiles(intent)

        setContent {
            CommonContent {
                ShareActivityScreen(vm)
            }
        }
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)

        intlHelper.updateMobileVaultIntlCurrentLocale(newConfig)
    }
}
