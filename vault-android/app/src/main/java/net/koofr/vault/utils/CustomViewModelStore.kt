package net.koofr.vault.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.lifecycle.HasDefaultViewModelProviderFactory
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.CreationExtras
import androidx.lifecycle.viewmodel.compose.LocalViewModelStoreOwner

@Composable
fun rememberCustomViewModelStore(store: ViewModelStore): ViewModelStoreOwner {
    val parentViewModelStoreOwner = checkNotNull(LocalViewModelStoreOwner.current) {
        "No ViewModelStoreOwner was provided via LocalViewModelStoreOwner"
    }

    val owner = remember(store, parentViewModelStoreOwner) {
        if (parentViewModelStoreOwner is HasDefaultViewModelProviderFactory) {
            object : ViewModelStoreOwner, HasDefaultViewModelProviderFactory {
                override val viewModelStore: ViewModelStore
                    get() = store
                override val defaultViewModelProviderFactory: ViewModelProvider.Factory
                    get() = parentViewModelStoreOwner.defaultViewModelProviderFactory
                override val defaultViewModelCreationExtras: CreationExtras
                    get() = parentViewModelStoreOwner.defaultViewModelCreationExtras
            }
        } else {
            object : ViewModelStoreOwner {
                override val viewModelStore: ViewModelStore get() = store
            }
        }
    }

    return owner
}

@Composable
fun WithCustomViewModelStore(store: ViewModelStore, content: @Composable () -> Unit) {
    val owner = rememberCustomViewModelStore(store)

    CompositionLocalProvider(LocalViewModelStoreOwner provides owner) {
        content()
    }
}
