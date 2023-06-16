package net.koofr.vault.features.uploads

import net.koofr.vault.SizeInfo
import net.koofr.vault.StreamException
import net.koofr.vault.UploadStream
import net.koofr.vault.UploadStreamProvider
import java.io.IOException
import java.io.InputStream

class UploadInputStreamProvider constructor(
    private val inputStream: InputStream,
    private val size: Long?,
) : UploadStreamProvider {
    private var streamCalled: Boolean = false

    override fun size(): SizeInfo {
        return if (size != null) {
            SizeInfo.Estimate(size)
        } else {
            SizeInfo.Unknown
        }
    }

    override fun isRetriable(): Boolean {
        return inputStream.markSupported()
    }

    override fun stream(): UploadStream {
        try {
            if (streamCalled) {
                inputStream.reset()
            }
        } catch (ex: IOException) {
            throw StreamException.NotRetriable()
        }

        streamCalled = true

        return UploadInputStream(inputStream)
    }

    override fun dispose() {
        try {
            inputStream.close()
        } catch (ex: IOException) {
            throw StreamException.IoException(ex.toString())
        }
    }
}
