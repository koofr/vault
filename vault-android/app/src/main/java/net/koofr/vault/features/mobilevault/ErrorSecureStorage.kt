package net.koofr.vault.features.mobilevault

import net.koofr.vault.SecureStorage

class ErrorSecureStorage(exception: Exception) : SecureStorage {
    private var exception: Exception = exception

    override fun getItem(key: String): String? {
        throw exception
    }

    override fun setItem(key: String, value: String) {
        throw exception
    }

    override fun removeItem(key: String) {
        throw exception
    }

    override fun clear() {
        throw exception
    }
}

