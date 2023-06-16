package net.koofr.vault.composables

import android.content.Context
import android.net.Uri
import android.view.ViewGroup.LayoutParams.MATCH_PARENT
import android.widget.FrameLayout
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import androidx.media3.ui.PlayerView

@androidx.annotation.OptIn(UnstableApi::class)
fun buildExoPlayer(context: Context, uri: Uri): ExoPlayer {
    return ExoPlayer.Builder(context)
        .build()
        .apply {
            val defaultDataSourceFactory = DefaultDataSource.Factory(context)
            val dataSourceFactory: DataSource.Factory = DefaultDataSource.Factory(
                context,
                defaultDataSourceFactory,
            )
            val source = ProgressiveMediaSource.Factory(dataSourceFactory)
                .createMediaSource(MediaItem.fromUri(uri))

            setMediaSource(source)
            prepare()
        }
}

@Composable
fun VideoPlayer(exoPlayer: ExoPlayer) {
    val context = LocalContext.current

    AndroidView(factory = {
        PlayerView(context).apply {
            player = exoPlayer
            layoutParams = FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT)
        }
    })
}
