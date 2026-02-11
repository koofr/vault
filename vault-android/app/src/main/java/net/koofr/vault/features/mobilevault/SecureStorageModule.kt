package net.koofr.vault.features.mobilevault

import android.content.Context
import android.util.Log
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.R
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
        return try {
            AndroidSecureStorage(appContext)
        } catch (e: Exception) {
            Log.e("Vault", "Failed to build android secure storage", e)

            ErrorSecureStorage(
                Exception(
                    appContext.getString(
                        R.string.secure_storage_read_failed_message,
                        e.toString(),
                    ),
                ),
            )
        }
    }
}
