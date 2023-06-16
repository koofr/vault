package net.koofr.vault.features.sharetarget

import net.koofr.vault.LocalFile
import net.koofr.vault.features.uploads.UploadFile

data class ShareTargetFile(val localFile: LocalFile, val uploadFile: UploadFile)
