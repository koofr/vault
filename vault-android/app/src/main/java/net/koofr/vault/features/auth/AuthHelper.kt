package net.koofr.vault.features.auth

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.MobileVault

class AuthHelper constructor(private val mobileVault: MobileVault) {
    fun login(
        context: Context,
    ) {
        val url = Uri.parse(mobileVault.oauth2StartLoginFlow() + "&platform=android")

        startFlow(context, url)
    }

    fun logout(
        context: Context,
    ) {
        val url = Uri.parse(mobileVault.oauth2StartLogoutFlow() + "&platform=android")

        startFlow(context, url)
    }

    private fun startFlow(
        context: Context,
        url: Uri,
    ) {
        val intent = CustomTabsIntent.Builder().build()
        val keepAliveIntent = Intent(context, KeepAliveService::class.java)

        intent.intent.addFlags(
            Intent.FLAG_ACTIVITY_SINGLE_TOP or
                Intent.FLAG_ACTIVITY_NEW_TASK or
                Intent.FLAG_FROM_BACKGROUND,
        )
        intent.intent.putExtra(
            "android.support.customtabs.extra.KEEP_ALIVE",
            keepAliveIntent,
        )
        intent.intent.putExtra(
            Intent.EXTRA_REFERRER,
            Uri.parse("android-app://" + context.packageName),
        )

        intent.launchUrl(context, url)
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object AuthHelperModule {
    @ActivityRetainedScoped
    @Provides
    fun provideAuthHelper(mobileVault: MobileVault): AuthHelper {
        return AuthHelper(mobileVault)
    }
}
