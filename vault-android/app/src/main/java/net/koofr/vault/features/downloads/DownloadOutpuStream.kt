package net.koofr.vault.features.downloads

import net.koofr.vault.DownloadStream
import net.koofr.vault.StreamException
import java.io.IOException
import java.io.OutputStream

class DownloadOutpuStream constructor(private val outputStream: OutputStream) : DownloadStream {
    override fun write(buf: ByteArray) {
        try {
            outputStream.write(buf)
        } catch (ex: IOException) {
            throw StreamException.IoException(ex.toString())
        }
    }

    override fun close() {
        try {
            outputStream.close()
        } catch (ex: IOException) {
            throw StreamException.IoException(ex.toString())
        }
    }
}
