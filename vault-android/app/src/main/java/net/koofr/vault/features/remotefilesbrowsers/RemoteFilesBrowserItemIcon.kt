package net.koofr.vault.features.remotefilesbrowsers

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import net.koofr.vault.FileIconProps
import net.koofr.vault.FileIconSize
import net.koofr.vault.MountOrigin
import net.koofr.vault.R
import net.koofr.vault.RemoteFilesBrowserItem
import net.koofr.vault.RemoteFilesBrowserItemType
import net.koofr.vault.features.fileicon.FileIconCache

@Composable
private fun VectorIcon(id: Int) {
    Icon(
        painter = painterResource(id = id),
        contentDescription = null,
        tint = Color.Unspecified,
        modifier = Modifier
            .padding(6.dp)
            .fillMaxSize(),
    )
}

@Composable
fun RemoteFilesBrowserItemIcon(item: RemoteFilesBrowserItem, fileIconCache: FileIconCache) {
    item.typ.let {
        when (it) {
            is RemoteFilesBrowserItemType.Bookmarks -> VectorIcon(R.drawable.ic_bookmarks)
            is RemoteFilesBrowserItemType.Place -> when (it.origin) {
                MountOrigin.HOSTED -> VectorIcon(R.drawable.ic_hosted)
                MountOrigin.DESKTOP -> VectorIcon(R.drawable.ic_desktop)
                MountOrigin.DROPBOX -> VectorIcon(R.drawable.ic_dropbox)
                MountOrigin.GOOGLEDRIVE -> VectorIcon(R.drawable.ic_googledrive)
                MountOrigin.ONEDRIVE -> VectorIcon(R.drawable.ic_onedrive)
                MountOrigin.SHARE -> VectorIcon(R.drawable.ic_shared)
                MountOrigin.OTHER -> VectorIcon(R.drawable.ic_hosted)
            }

            is RemoteFilesBrowserItemType.File -> Image(
                fileIconCache.getIcon(
                    FileIconProps(
                        size = FileIconSize.SM,
                        attrs = it.fileIconAttrs,
                    ),
                ),
                null,
            )

            is RemoteFilesBrowserItemType.Shared -> VectorIcon(R.drawable.ic_shared)
        }
    }
}
