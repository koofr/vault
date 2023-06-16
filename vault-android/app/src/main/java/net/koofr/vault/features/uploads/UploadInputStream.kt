package net.koofr.vault.features.uploads

import net.koofr.vault.StreamException
import net.koofr.vault.UploadStream
import java.io.IOException
import java.io.InputStream

class UploadInputStream constructor(private val inputStream: InputStream) : UploadStream {
    override fun read(): ByteArray {
        try {
            val buf = ByteArray(1024 * 1024)
            val n = inputStream.read(buf)

            return if (n > 0) {
                buf.copyOf(n)
            } else {
                byteArrayOf()
            }
        } catch (ex: IOException) {
            throw StreamException.IoException(ex.toString())
        }
    }

    override fun close() {
        // do not close input stream here, it will be closed in
        // UploadInputStreamProvider.dispose
    }
}
