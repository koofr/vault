package net.koofr.vault.tests

import net.koofr.vault.SecureStorage

class MemorySecureStorage : SecureStorage {
    private var data: HashMap<String, String> = HashMap()

    fun getData(): HashMap<String, String> {
        return HashMap(data)
    }

    @Synchronized
    override fun getItem(key: String): String? {
        return data[key]
    }

    @Synchronized
    override fun setItem(key: String, value: String) {
        data[key] = value
    }

    @Synchronized
    override fun removeItem(key: String) {
        data.remove(key)
    }

    @Synchronized
    override fun clear() {
        data.clear()
    }
}
