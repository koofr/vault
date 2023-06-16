package net.koofr.vault.features.repo

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.biometric.BiometricPrompt
import net.koofr.vault.features.mobilevault.AndroidSecureStorage
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.IvParameterSpec

class RepoPasswordBiometricsHelper constructor(
    private val repoId: String,
    private val androidSecureStorage: AndroidSecureStorage,
) {
    private val keyName = "vaultRepoPassword_${repoId}_v1"

    val promptInfo = BiometricPrompt.PromptInfo.Builder()
        .setTitle("Safe Key biometrics")
        .setSubtitle("Use biometrics to save your Safe Key")
        .setNegativeButtonText("Don't use biometrics")
        .build()

    fun isBiometricUnlockEnabled(): Boolean {
        return androidSecureStorage.getItem(
            keyName,
        ) != null
    }

    fun enableBiometricUnlock(cipher: Cipher, password: String) {
        val encryptedPassword = cipher.doFinal(
            password.toByteArray(Charsets.UTF_8),
        )
        val encryptedPasswordStr =
            Base64.encodeToString(
                encryptedPassword,
                Base64.DEFAULT,
            )
        val ivStr =
            Base64.encodeToString(
                cipher.iv,
                Base64.DEFAULT,
            )
        val value = "$ivStr|$encryptedPasswordStr"

        androidSecureStorage.setItem(
            keyName,
            value,
        )
    }

    fun getEncryptedPasswordIv(): Pair<ByteArray, ByteArray>? {
        return androidSecureStorage.getItem(
            keyName,
        )?.let { value ->
            val valueParts = value.split('|')
            val ivStr = valueParts[0]
            val iv = Base64.decode(ivStr, Base64.DEFAULT)
            val encryptedPasswordStr = valueParts[1]
            val encryptedPassword = Base64.decode(encryptedPasswordStr, Base64.DEFAULT)

            Pair(encryptedPassword, iv)
        }
    }

    fun decryptPassword(cipher: Cipher, encryptedPassword: ByteArray): String {
        val passwordBytes = cipher.doFinal(
            encryptedPassword,
        )

        return String(passwordBytes, Charsets.UTF_8)
    }

    fun removeBiometricUnlock() {
        androidSecureStorage.removeItem(
            keyName,
        )
    }

    private fun tryGetEncryptCryptoObject(): BiometricPrompt.CryptoObject? {
        return ensureSecretKey(keyName).let { secretKey ->
            val cipher = getCipher()

            cipher.init(Cipher.ENCRYPT_MODE, secretKey)

            BiometricPrompt.CryptoObject(cipher)
        }
    }

    fun getEncryptCryptoObject(): BiometricPrompt.CryptoObject? {
        return try {
            tryGetEncryptCryptoObject()
        } catch (e: KeyPermanentlyInvalidatedException) {
            // fingerprint has changed or has been removed
            removeSecretKey(keyName)

            tryGetEncryptCryptoObject()
        }
    }

    fun getDecryptCryptoObject(iv: ByteArray): BiometricPrompt.CryptoObject? {
        return loadSecretKey(keyName).let { secretKey ->
            val cipher = getCipher()

            cipher.init(Cipher.DECRYPT_MODE, secretKey, IvParameterSpec(iv))

            BiometricPrompt.CryptoObject(cipher)
        }
    }

    private fun getCipher(): Cipher {
        return Cipher.getInstance(
            "${KeyProperties.KEY_ALGORITHM_AES}/${KeyProperties.BLOCK_MODE_CBC}/${KeyProperties.ENCRYPTION_PADDING_PKCS7}",
        )
    }

    private fun generateSecretKey(keyGenParameterSpec: KeyGenParameterSpec) {
        val keyGenerator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            "AndroidKeyStore",
        )
        keyGenerator.init(keyGenParameterSpec)
        keyGenerator.generateKey()
    }

    private fun loadSecretKey(keyName: String): SecretKey? {
        val keyStore = KeyStore.getInstance("AndroidKeyStore")
        keyStore.load(null)
        return keyStore.getKey(keyName, null)?.let { it as SecretKey }
    }

    private fun removeSecretKey(keyName: String) {
        val keyStore = KeyStore.getInstance("AndroidKeyStore")
        keyStore.load(null)
        keyStore.deleteEntry(keyName)
    }

    private fun ensureSecretKey(keyName: String): SecretKey? {
        val secretKey = loadSecretKey(keyName)

        if (secretKey != null) {
            return secretKey
        }

        generateSecretKey(
            KeyGenParameterSpec.Builder(
                keyName,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            ).setBlockModes(KeyProperties.BLOCK_MODE_CBC)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_PKCS7)
                .setUserAuthenticationRequired(true).build(),
        )

        return loadSecretKey(keyName)
    }
}
