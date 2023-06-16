package net.koofr.vault.features.shareactivity

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.fragment.app.FragmentActivity
import dagger.hilt.android.AndroidEntryPoint
import net.koofr.vault.CommonContent
import net.koofr.vault.features.mobilevault.MobileVaultProvider
import javax.inject.Inject

@AndroidEntryPoint
class ShareActivity : FragmentActivity() {
    @Inject
    lateinit var mobileVaultProvider: MobileVaultProvider

    private val vm: ShareActivityViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        mobileVaultProvider.loadConfigFromIntent(intent)

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
}
