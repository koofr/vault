package net.koofr.vault.features.mobilevault

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.SecureStorage
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object SecureStorageModule {
    @Singleton
    @Provides
    fun provideSecureStorage(
        @ApplicationContext appContext: Context,
    ): SecureStorage {
        return AndroidSecureStorage(appContext)
    }
}