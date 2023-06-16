package net.koofr.vault.features.uploads

import java.io.InputStream

sealed class UploadFileData {
    class Stream(val stream: InputStream) : UploadFileData()
    class Bytes(val bytes: ByteArray) : UploadFileData()
}

data class UploadFile(val name: String, val size: Long?, val data: UploadFileData)
