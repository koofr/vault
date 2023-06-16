package net.koofr.vault

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.fragment.app.FragmentActivity
import dagger.hilt.android.AndroidEntryPoint
import net.koofr.vault.features.auth.AuthGuard
import net.koofr.vault.features.mobilevault.MobileVaultProvider
import net.koofr.vault.features.mobilevault.isMobileVaultSupported
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : FragmentActivity() {
    @Inject
    lateinit var mobileVaultProvider: MobileVaultProvider

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        mobileVaultProvider.loadConfigFromIntent(intent)

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
}
