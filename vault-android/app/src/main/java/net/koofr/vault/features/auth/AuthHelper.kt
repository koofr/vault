package net.koofr.vault.features.auth

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Log
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
        mobileVault.oauth2StartLoginFlow()?.let {
            val url = Uri.parse("$it&platform=android")

            startFlow(context, url)
        }
    }

    fun logout(
        context: Context,
    ) {
        mobileVault.oauth2StartLogoutFlow()?.let {
            val url = Uri.parse("$it&platform=android")

            startFlow(context, url)
        }
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

        try {
            intent.launchUrl(context, url)
        } catch (e: Exception) {
            Log.e("Vault", "Failed to launch custom tab for auth", e)

            mobileVault.notificationsShow("Failed to launch a browser app. Please make sure you have a web browser app installed.")
        }
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
