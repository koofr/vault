package net.koofr.vault.features.mobilevault

import android.util.Log
import net.koofr.vault.LoggerCallback
import net.koofr.vault.LoggerLevel

class AndroidLogger : LoggerCallback {
    companion object {
        private const val TAG = "Vault"
    }

    override fun log(level: LoggerLevel, message: String) {
        when (level) {
            LoggerLevel.ERROR -> Log.e(TAG, message)
            LoggerLevel.WARN -> Log.w(TAG, message)
            LoggerLevel.INFO -> Log.i(TAG, message)
            LoggerLevel.DEBUG -> Log.d(TAG, message)
            LoggerLevel.TRACE -> Log.v(TAG, message)
        }
    }
}
