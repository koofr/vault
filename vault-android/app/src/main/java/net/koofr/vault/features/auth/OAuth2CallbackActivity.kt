package net.koofr.vault.features.auth

import android.os.Bundle
import androidx.activity.ComponentActivity
import dagger.hilt.android.AndroidEntryPoint
import net.koofr.vault.MobileVault
import net.koofr.vault.OAuth2FinishFlowDone
import javax.inject.Inject

@AndroidEntryPoint
class OAuth2CallbackActivity : ComponentActivity() {
    @Inject
    lateinit var mobileVault: MobileVault

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        intent?.data?.let { url ->
            mobileVault.oauth2FinishFlowUrl(
                url = url.toString(),
                cb = object : OAuth2FinishFlowDone {
                    override fun onDone() {}
                },
            )
        }

        finish()
    }
}
