package net.koofr.vault.features.mobilevault

import androidx.compose.runtime.mutableStateOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.koofr.vault.MobileVault
import net.koofr.vault.SubscriptionCallback
import java.io.Closeable

class Subscription<T> constructor(
    private val mobileVault: MobileVault,
    private val coroutineScope: CoroutineScope,
    subscribe: (MobileVault, SubscriptionCallback) -> UInt,
    private val getData: (MobileVault, UInt) -> T?,
) : Closeable {
    private var id: UInt? = null
    val data = mutableStateOf<T?>(null)
    private var onData: ((T?) -> Unit)? = null

    init {
        id = subscribe(
            mobileVault,
            object : SubscriptionCallback {
                override fun onChange(id: UInt) {
                    coroutineScope.launch {
                        update(id)
                    }
                }
            },
        )

        data.value = getData(mobileVault, id!!)
    }

    private fun update(id: UInt) {
        val data = getData(mobileVault, id)

        this.data.value = data

        onData?.let { it(data) }
    }

    fun setOnData(onData: (T?) -> Unit) {
        this.onData = onData

        onData(this.data.value)
    }

    override fun close() {
        id?.let {
            mobileVault.unsubscribe(id = it)
        }
    }
}
