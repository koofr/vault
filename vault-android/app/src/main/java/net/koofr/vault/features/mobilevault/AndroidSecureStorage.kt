package net.koofr.vault.features.mobilevault

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import androidx.security.crypto.MasterKey.Builder
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.SecureStorage
import javax.inject.Singleton

class AndroidSecureStorage : SecureStorage {
    private var sharedPreferences: SharedPreferences

    companion object {
        const val SHARED_PREFERENCES_NAME = "VaultSecureStorage"
    }

    constructor(context: Context) {
        val masterKey: MasterKey = Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        sharedPreferences = EncryptedSharedPreferences.create(
            context,
            SHARED_PREFERENCES_NAME,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    override fun getItem(key: String): String? {
        return sharedPreferences.getString(key, null)
    }

    override fun setItem(key: String, value: String) {
        val editor = sharedPreferences.edit()
        editor.putString(key, value)
        editor.apply()
    }

    override fun removeItem(key: String) {
        val editor = sharedPreferences.edit()
        editor.remove(key)
        editor.apply()
    }

    override fun clear() {
        val editor = sharedPreferences.edit()
        editor.clear()
        editor.apply()
    }
}

@Module
@InstallIn(SingletonComponent::class)
object AndroidSecureStorageModule {
    @Singleton
    @Provides
    fun provideAndroidSecureStorage(
        @ApplicationContext appContext: Context,
    ): AndroidSecureStorage {
        return AndroidSecureStorage(appContext)
    }
}
