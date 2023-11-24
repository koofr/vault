package net.koofr.vault.features.transfers

import androidx.navigation.NavController
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.scopes.ActivityRetainedScoped
import kotlinx.coroutines.MainScope
import net.koofr.vault.MobileVault
import net.koofr.vault.features.mobilevault.Subscription
import java.io.Closeable

class TransfersHelper(mobileVault: MobileVault) : Closeable {
    private val transfersIsActiveSubscription = Subscription(
        mobileVault = mobileVault,
        coroutineScope = MainScope(),
        subscribe = { v, cb -> v.transfersIsActiveSubscribe(cb = cb) },
        getData = { v, id -> v.transfersIsActiveData(id = id) },
    )
    private var navControllerWhenActivate: NavController? = null

    init {
        transfersIsActiveSubscription.setOnData {
            if (it == true) {
                navControllerWhenActivate?.let { navController ->
                    navControllerWhenActivate = null

                    navController.navigate("transfers")
                }
            }
        }
    }

    fun navigateWhenActive(navController: NavController) {
        if (transfersIsActiveSubscription.data.value == true) {
            navController.navigate("transfers")
        } else {
            navControllerWhenActivate = navController
        }
    }

    override fun close() {
        transfersIsActiveSubscription.close()
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object TransfersHelperModule {
    @ActivityRetainedScoped
    @Provides
    fun provideTransfersHelper(
        mobileVault: MobileVault,
    ): TransfersHelper {
        return TransfersHelper(mobileVault)
    }
}
