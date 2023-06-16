package net.koofr.vault.features.mobilevault

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.MobileVault

@Module
@InstallIn(ActivityRetainedComponent::class)
object ActivityMobileVaultProviderModule {
    @ActivityRetainedScoped
    @Provides
    fun provideConfig(
        mobileVaultProvider: MobileVaultProvider,
    ): Config {
        return mobileVaultProvider.getConfig()
    }

    @ActivityRetainedScoped
    @Provides
    fun provideMobileVault(
        mobileVaultProvider: MobileVaultProvider,
    ): MobileVault {
        return mobileVaultProvider.getMobileVault()
    }
}
