package net.koofr.vault.tests.helpers

import net.koofr.vault.FileCategory
import net.koofr.vault.FilesFilter
import net.koofr.vault.MobileVault
import net.koofr.vault.Repo
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesBrowserOptions
import net.koofr.vault.RepoFilesBrowserSource
import net.koofr.vault.RepoFilesDetailsOptions
import net.koofr.vault.RepoUnlockMode
import net.koofr.vault.RepoUnlockOptions
import net.koofr.vault.RepoUnlockUnlocked
import net.koofr.vault.Status
import net.koofr.vault.SubscriptionCallback
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException
import java.util.concurrent.locks.ReentrantLock

class MobileVaultHelper constructor(private val mobileVault: MobileVault) {
    fun <T> subscriptionWait(
        subscribe: (MobileVault, SubscriptionCallback) -> UInt,
        getData: (MobileVault, UInt) -> T?,
        timeoutMs: Long = 10000,
    ): T {
        var id: UInt? = null
        val lock = ReentrantLock()
        val condition = lock.newCondition()
        var callbackData: T? = null

        id = subscribe(
            mobileVault,
            object : SubscriptionCallback {
                override fun onChange(id: UInt) {
                    val data = getData(mobileVault, id)

                    data?.let {
                        lock.lock()

                        try {
                            mobileVault.unsubscribe(id = id)

                            callbackData = it

                            condition.signal()
                        } finally {
                            lock.unlock()
                        }
                    }
                }
            },
        )

        val data = getData(mobileVault, id)

        if (data != null) {
            mobileVault.unsubscribe(id = id)

            return data
        }

        lock.lock()

        try {
            var remainingNanos = TimeUnit.MILLISECONDS.toNanos(timeoutMs)
            while (callbackData == null) {
                if (remainingNanos <= 0) {
                    throw TimeoutException("subscriptionWait wait timed out")
                }
                remainingNanos = condition.awaitNanos(remainingNanos)
            }

            return callbackData
        } finally {
            lock.unlock()
        }
    }

    fun waitForOAuth2Loaded() {
        subscriptionWait(
            { v, cb -> v.oauth2StatusSubscribe(cb = cb) },
            { v, id ->
                v.oauth2StatusData(id = id)?.takeIf { it is Status.Loaded }
            },
        )
    }

    fun waitForReposLoaded() {
        subscriptionWait(
            { v, cb -> v.reposSubscribe(cb = cb) },
            { v, id -> v.reposData(id = id)?.takeIf { it.status is Status.Loaded } },
        )
    }

    fun waitForRepo(): Repo {
        return subscriptionWait(
            { v, cb -> v.reposSubscribe(cb = cb) },
            { v, id ->
                v.reposData(id = id)?.takeIf { it.status is Status.Loaded }
                    ?.let { it.repos.firstOrNull() }
            },
        )
    }

    fun unlockRepo(repo: Repo, password: String = "password") {
        val unlockId =
            mobileVault.repoUnlockCreate(repoId = repo.id, options = RepoUnlockOptions(mode = RepoUnlockMode.UNLOCK))

        try {
            mobileVault.repoUnlockUnlock(
                unlockId = unlockId,
                password = password,
                cb = object : RepoUnlockUnlocked {
                    override fun onUnlocked() {}
                },
            )

            subscriptionWait(
                { v, cb -> v.repoUnlockInfoSubscribe(unlockId = unlockId, cb = cb) },
                { v, id -> v.repoUnlockInfoData(id = id)?.takeIf { it.status is Status.Loaded } },
            )
        } finally {
            mobileVault.repoUnlockDestroy(unlockId = unlockId)
        }
    }

    fun waitForRepoUnlock(password: String = "password"): Repo {
        val repo = waitForRepo()

        unlockRepo(repo, password)

        return repo
    }

    fun encryptName(repo: Repo, name: String): String {
        return mobileVault.repoFilesEncryptName(repoId = repo.id, name = name)
            ?: throw Exception("Failed to encrypt name: $name")
    }

    fun uploadFile(repo: Repo, encryptedParentPath: String, name: String, content: String): RepoFile {
        val browserId = mobileVault.repoFilesBrowsersCreate(
            source = RepoFilesBrowserSource.Storage(repoId = repo.id, encryptedPath = encryptedParentPath),
            options = RepoFilesBrowserOptions(selectName = null),
        )

        try {
            mobileVault.transfersUploadBytes(repoId = repo.id, encryptedParentPath = encryptedParentPath, name, content.toByteArray())

            return subscriptionWait(
                { v, cb -> v.repoFilesBrowsersInfoSubscribe(browserId = browserId, cb = cb) },
                { v, id -> v.repoFilesBrowsersInfoData(id = id)?.items?.find { it.file.name == name }?.file },
            )
        } finally {
            mobileVault.repoFilesBrowsersDestroy(browserId = browserId)
        }
    }

    fun getFileContent(repo: Repo, encryptedPath: String): String {
        val detailsId = mobileVault.repoFilesDetailsCreate(
            repoId = repo.id,
            encryptedPath = encryptedPath,
            isEditing = false,
            options = RepoFilesDetailsOptions(
                loadContent = FilesFilter(categories = listOf(FileCategory.CODE, FileCategory.TEXT), exts = emptyList()),
                autosaveIntervalMs = 20000u,
            ),
        )
        try {
            return subscriptionWait(
                { v, cb -> v.repoFilesDetailsContentBytesSubscribe(detailsId = detailsId, cb = cb) },
                { v, id ->
                    v.repoFilesDetailsContentBytesData(id = id)?.let { data ->
                        String(data, Charsets.UTF_8)
                    }
                },
            )
        } finally {
            mobileVault.repoFilesDetailsDestroy(detailsId = detailsId)
        }
    }

    fun waitForFileContent(repo: Repo, encryptedPath: String, expectedContent: String) {
        val detailsId = mobileVault.repoFilesDetailsCreate(
            repoId = repo.id,
            encryptedPath = encryptedPath,
            isEditing = false,
            options = RepoFilesDetailsOptions(
                loadContent = FilesFilter(categories = listOf(FileCategory.CODE, FileCategory.TEXT), exts = emptyList()),
                autosaveIntervalMs = 20000u,
            ),
        )
        try {
            subscriptionWait(
                { v, cb -> v.repoFilesDetailsContentBytesSubscribe(detailsId = detailsId, cb = cb) },
                { v, id ->
                    v.repoFilesDetailsContentBytesData(id = id)?.let { data ->
                        String(data, Charsets.UTF_8).takeIf { it == expectedContent }?.let { Unit }
                    }
                },
            )
        } finally {
            mobileVault.repoFilesDetailsDestroy(detailsId = detailsId)
        }
    }

    fun setFileContent(repo: Repo, encryptedPath: String, content: String) {
        val detailsId = mobileVault.repoFilesDetailsCreate(
            repoId = repo.id,
            encryptedPath = encryptedPath,
            isEditing = true,
            options = RepoFilesDetailsOptions(
                loadContent = FilesFilter(categories = listOf(FileCategory.CODE, FileCategory.TEXT), exts = emptyList()),
                autosaveIntervalMs = 20000u,
            ),
        )
        try {
            // wait for content loaded
            subscriptionWait(
                { v, cb -> v.repoFilesDetailsInfoSubscribe(detailsId = detailsId, cb = cb) },
                { v, id ->
                    v.repoFilesDetailsInfoData(id = id)?.takeIf { it.contentStatus is Status.Loaded }?.let { Unit }
                },
            )
            mobileVault.repoFilesDetailsSetContent(
                detailsId = detailsId,
                content = content.toByteArray(Charsets.UTF_8),
            )
            mobileVault.repoFilesDetailsSave(detailsId = detailsId)
            // wait for saved
            subscriptionWait(
                { v, cb -> v.repoFilesDetailsInfoSubscribe(detailsId = detailsId, cb = cb) },
                { v, id ->
                    v.repoFilesDetailsInfoData(id = id)?.takeIf { !it.isDirty }?.let { Unit }
                },
            )
        } finally {
            mobileVault.repoFilesDetailsDestroy(detailsId = detailsId)
        }
    }

    fun ensureFile(repo: Repo, encryptedPath: String) {
        val (encryptedParentPath, _) = splitParentName(encryptedPath)
        val browserId = mobileVault.repoFilesBrowsersCreate(
            source = RepoFilesBrowserSource.Storage(repoId = repo.id, encryptedPath = encryptedParentPath),
            options = RepoFilesBrowserOptions(selectName = null),
        )
        try {
            subscriptionWait(
                { v, cb -> v.repoFilesBrowsersInfoSubscribe(browserId = browserId, cb = cb) },
                { v, id ->
                    v.repoFilesBrowsersInfoData(id = id)?.takeIf { it.status is Status.Loaded }?.let { Unit }
                },
            )
        } finally {
            mobileVault.repoFilesBrowsersDestroy(browserId = browserId)
        }
    }

    fun renameFile(repo: Repo, encryptedPath: String, newName: String) {
        ensureFile(repo, encryptedPath)
        mobileVault.repoFilesRenameFile(repoId = repo.id, encryptedPath = encryptedPath)
        promptDialogFill(newName)
    }

    fun deleteFile(repo: Repo, encryptedPath: String) {
        ensureFile(repo, encryptedPath)
        mobileVault.repoFilesDeleteFile(repoId = repo.id, encryptedPath = encryptedPath)
        confirmDialog()
    }

    fun promptDialogFill(value: String) {
        confirmDialog(inputValue = value)
    }

    fun confirmDialog(inputValue: String? = null) {
        // get first dialog id
        val dialogId = subscriptionWait(
            { v, cb -> v.dialogsSubscribe(cb = cb) },
            { v, id ->
                v.dialogsData(id = id)?.takeIf { it.isNotEmpty() }?.get(0)
            },
        )
        // wait for the dialog to exist
        subscriptionWait(
            { v, cb -> v.dialogsDialogSubscribe(dialogId = dialogId, cb = cb) },
            { v, id -> v.dialogsDialogData(id = id) },
        )
        // if input value is set, set the value
        inputValue?.let {
            mobileVault.dialogsSetInputValue(dialogId = dialogId, value = it)
        }
        // confirm the dialog
        mobileVault.dialogsConfirm(dialogId = dialogId)
        // wait for the dialog to be destroyed
        subscriptionWait(
            { v, cb -> v.dialogsDialogSubscribe(dialogId = dialogId, cb = cb) },
            { v, id -> if (v.dialogsDialogData(id = id) == null) Unit else null },
        )
    }
}
