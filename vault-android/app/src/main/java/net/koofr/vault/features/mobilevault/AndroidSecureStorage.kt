package net.koofr.vault.features.mobilevault

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import androidx.security.crypto.MasterKey.Builder
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.koofr.vault.SecureStorage
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.inject.Singleton

class AndroidSecureStorage(context: Context) : SecureStorage {
    private var sharedPreferences: SharedPreferences

    companion object {
        private const val SHARED_PREFERENCES_NAME = "VaultSecureStorage"

        private fun ensureEncryptedSharedPreferences(
            context: Context,
        ): SharedPreferences {
            Log.i("Vault", "Ensuring encrypted shared preferences")

            return try {
                createEncryptedSharedPreferences(context)
            } catch (e: GeneralSecurityException) {
                Log.e("Vault", "Failed to create encrypted shared preferences", e)

                Log.i("Vault", "Cleaning up master key and shared preferences")

                cleanupEncryptedSharedPreferences(context)

                Log.i("Vault", "Retrying create encrypted shared preferences")

                createEncryptedSharedPreferences(context)
            }
        }

        private fun createEncryptedSharedPreferences(
            context: Context,
        ): SharedPreferences {
            val masterKey: MasterKey = Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()

            return EncryptedSharedPreferences.create(
                context,
                SHARED_PREFERENCES_NAME,
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
            )
        }

        private fun cleanupEncryptedSharedPreferences(context: Context) {
            tryCleanupMasterKey()
            tryCleanupSharedPreferences(context)
        }

        private fun tryCleanupMasterKey() {
            try {
                val keyStore = KeyStore.getInstance("AndroidKeyStore")
                keyStore.load(null)
                keyStore.deleteEntry(MasterKey.DEFAULT_MASTER_KEY_ALIAS)
            } catch (e: Exception) {
                Log.e("Vault", "Failed to cleanup master key", e)
            }
        }

        private fun tryCleanupSharedPreferences(context: Context) {
            try {
                context.getSharedPreferences(SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE).edit()
                    .clear().apply()
            } catch (e: Exception) {
                Log.e("Vault", "Failed to cleanup shared preferences", e)
            }
        }
    }

    init {
        sharedPreferences = ensureEncryptedSharedPreferences(context)
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
