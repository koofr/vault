package net.koofr.vault.composables

import android.view.ViewGroup
import android.widget.ImageView
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import coil.ImageLoader
import coil.request.ImageRequest
import com.github.chrisbanes.photoview.PhotoView
import java.io.File

@Composable
fun ZoomableImage(
    imageFile: File,
    ext: String,
    imageLoader: ImageLoader,
    modifier: Modifier = Modifier,
) {
    key(imageFile.absolutePath) {
        AndroidView(
            modifier = modifier,
            factory = { context ->
                val view = PhotoView(context)

                view.maximumScale = 10.0f

                view.scaleType = ImageView.ScaleType.CENTER_INSIDE

                view.layoutParams = ViewGroup.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT,
                )

                val request = ImageRequest.Builder(context)
                    .data(imageFile)
                    .target(view)
                    .build()

                imageLoader.enqueue(request)

                view

//                val view = SubsamplingScaleImageView(context)
//
//                view.layoutParams = ViewGroup.LayoutParams(
//                    ViewGroup.LayoutParams.MATCH_PARENT,
//                    ViewGroup.LayoutParams.MATCH_PARENT
//                )
//
//                if (ext == "jpg" || ext == "jpeg" || ext == "heic") {
//                    view.setImage(ImageSource.uri(imageFile.toUri()))
//                } else {
//                    val request = ImageRequest.Builder(context)
//                        .data(imageFile)
//                        .target(onSuccess = {
//                            view.setImage(ImageSource.cachedBitmap(it.toBitmap()))
//                        })
//                        .build()
//
//                    imageLoader.enqueue(request)
//                }
//
//                view
            },
        )
    }
}
