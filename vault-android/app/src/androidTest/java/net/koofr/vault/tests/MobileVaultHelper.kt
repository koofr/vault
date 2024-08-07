package net.koofr.vault.tests

import net.koofr.vault.MobileVault
import net.koofr.vault.Repo
import net.koofr.vault.RepoFile
import net.koofr.vault.RepoFilesBrowserOptions
import net.koofr.vault.RepoUnlockMode
import net.koofr.vault.RepoUnlockOptions
import net.koofr.vault.RepoUnlockUnlocked
import net.koofr.vault.Status
import net.koofr.vault.SubscriptionCallback
import java.util.concurrent.locks.ReentrantLock

class MobileVaultHelper constructor(private val mobileVault: MobileVault) {
    fun <T> subscriptionWait(
        subscribe: (MobileVault, SubscriptionCallback) -> UInt,
        getData: (MobileVault, UInt) -> T?,
    ): T {
        var id: UInt? = null
        val lock = ReentrantLock()
        val condition = lock.newCondition()
        var callbackData: T? = null

        id = subscribe(
            mobileVault,
            object : SubscriptionCallback {
                override fun onChange() {
                    id?.let { id ->
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
            while (callbackData == null) {
                condition.await()
            }

            return callbackData!!
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

    fun uploadFile(repo: Repo, encryptedParentPath: String, name: String, content: String): RepoFile {
        val browserId = mobileVault.repoFilesBrowsersCreate(
            repoId = repo.id,
            encryptedPath = encryptedParentPath,
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
}
