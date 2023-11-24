package net.koofr.vault.features.fileicon

import android.graphics.BitmapFactory
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ActivityRetainedComponent
import dagger.hilt.android.scopes.ActivityRetainedScoped
import net.koofr.vault.FileIconProps
import net.koofr.vault.MobileVault
import javax.inject.Inject

data class FileIconCacheKey(val props: FileIconProps, val scale: Int)

class FileIconCache @Inject constructor(val mobileVault: MobileVault) {
    private val cache: MutableMap<FileIconCacheKey, ImageBitmap> = mutableMapOf()

    @Synchronized
    fun getIcon(props: FileIconProps, scale: Int = 3): ImageBitmap {
        val key = FileIconCacheKey(props, scale)

        val cachedIcon = cache[key]

        return if (cachedIcon != null) {
            cachedIcon
        } else {
            val icon = buildIcon(props, scale)
            cache[key] = icon
            icon
        }
    }

    private fun buildIcon(props: FileIconProps, scale: Int): ImageBitmap {
        val png = mobileVault.fileIconPng(props = props, scale = scale.toUInt())

        val pngByteArray = png.png

        val bitmap = BitmapFactory.decodeByteArray(pngByteArray, 0, pngByteArray.size)

        return bitmap.asImageBitmap()
    }
}

@Module
@InstallIn(ActivityRetainedComponent::class)
object FileIconCacheModule {
    @ActivityRetainedScoped
    @Provides
    fun provideFileIconCache(mobileVault: MobileVault): FileIconCache {
        return FileIconCache(mobileVault)
    }
}
