package net.koofr.vault.features.intl

import android.content.res.Configuration
import android.util.Log
import androidx.appcompat.app.AppCompatDelegate
import androidx.core.os.LocaleListCompat
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.IntlChangeLocaleStrategy
import net.koofr.vault.features.mobilevault.MobileVaultProvider
import java.util.Locale

/**
 * Must depend on [MobileVaultProvider], not MobileVault directly.
 *
 * Injecting MobileVault here initializes it too early, before
 * `MainActivity.mobileVaultProvider.loadConfigFromIntent()` runs. In tests,
 * intent data is used there to override MobileVault config. Early
 * initialization breaks that flow.
 */
class IntlHelper constructor(private val mobileVaultProvider: MobileVaultProvider) {
    fun updateMobileVaultIntlCurrentLocale(configuration: Configuration) {
        val locales = getCurrentLocales(configuration)

        mobileVaultProvider.getMobileVault().intlChangeLocale(
            strategy = IntlChangeLocaleStrategy.Lookup(
                locales = locales.map {
                    it.stripExtensions().toLanguageTag()
                },
            ),
        )
    }

    private fun getCurrentLocales(configuration: Configuration): List<Locale> {
        val appLocales = AppCompatDelegate.getApplicationLocales()

        return if (!appLocales.isEmpty) {
            (0 until appLocales.size())
                .mapNotNull { appLocales[it] }
        } else {
            val locales = configuration.locales

            (0 until locales.size())
                .map { locales[it] }
        }
    }

    fun changeLocale(languageTag: String) {
        // mobileVault.intlChangeLocale() is not called here, it's called by
        // updateMobileVaultIntlCurrentLocale() when the configuration changes
        // and the activity is recreated
        val locales = LocaleListCompat.forLanguageTags(languageTag)

        AppCompatDelegate.setApplicationLocales(locales)
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object IntlHelperModule {
    @ActivityRetainedScoped
    @Provides
    fun provideIntlHelper(mobileVaultProvider: MobileVaultProvider): IntlHelper {
        return IntlHelper(mobileVaultProvider)
    }
}
