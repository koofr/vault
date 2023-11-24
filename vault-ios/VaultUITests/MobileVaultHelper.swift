import Foundation
import VaultMobile

class MobileVaultHelper {
    let mobileVault: MobileVault

    init(mobileVault: MobileVault) {
        self.mobileVault = mobileVault
    }

    func waitForOAuth2Loaded() async {
        await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.oauth2StatusSubscribe(cb: cb)
            },
            getData: { v, id in
                let data = v.oauth2StatusData(id: id)
                return data != nil && data == .loaded ? () : nil
            }
        )
    }

    func waitForReposLoaded() async {
        await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.reposSubscribe(cb: cb)
            },
            getData: { v, id in
                let data = v.reposData(id: id)
                return data != nil && data!.status == .loaded ? () : nil
            }
        )
    }

    func waitForRepo() async -> Repo {
        await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.reposSubscribe(cb: cb)
            },
            getData: { v, id in
                let data = v.reposData(id: id)
                return data != nil && data!.status == .loaded && data!.repos.count > 0
                    ? data!.repos[0] : nil
            }
        )
    }

    func unlockRepo(repo: Repo, password: String = "password") async {
        let unlockId = mobileVault.repoUnlockCreate(
            repoId: repo.id, options: RepoUnlockOptions(mode: .unlock))

        defer {
            mobileVault.repoUnlockDestroy(unlockId: unlockId)
        }

        mobileVault.repoUnlockUnlock(
            unlockId: unlockId, password: password, cb: RepoUnlockUnlockedFn {})

        await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoUnlockInfoSubscribe(unlockId: unlockId, cb: cb)
            },
            getData: { v, id in
                let data = v.repoUnlockInfoData(id: id)
                return data != nil && data!.status == .loaded ? () : nil
            }
        )
    }

    func waitForRepoUnlock(password: String = "password") async -> Repo {
        let repo = await waitForRepo()

        await unlockRepo(repo: repo, password: password)

        return repo
    }

    func uploadFile(repo: Repo, encryptedParentPath: String, name: String, content: String) async
        -> RepoFile
    {
        let browserId = mobileVault.repoFilesBrowsersCreate(
            repoId: repo.id, encryptedPath: encryptedParentPath,
            options: RepoFilesBrowserOptions(selectName: nil))

        defer {
            mobileVault.repoFilesBrowsersDestroy(browserId: browserId)
        }

        mobileVault.transfersUploadBytes(
            repoId: repo.id, encryptedParentPath: encryptedParentPath, name: name,
            bytes: content.data(using: .utf8)!)

        return await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesBrowsersInfoSubscribe(browserId: browserId, cb: cb)
            },
            getData: { v, id in
                let data = v.repoFilesBrowsersInfoData(id: id)
                if let info = data {
                    return info.items.first(where: { $0.file.name == name }).map { $0.file }
                } else {
                    return nil
                }
            }
        )
    }
}
