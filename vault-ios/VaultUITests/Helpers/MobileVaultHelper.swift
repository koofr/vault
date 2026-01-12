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

    func encryptName(repo: Repo, name: String) throws -> String {
        guard
            let encryptedName = mobileVault.repoFilesEncryptName(
                repoId: repo.id, name: name)
        else {
            throw NSError(
                domain: "EncryptionError", code: 0,
                userInfo: [NSLocalizedDescriptionKey: "Failed to encrypt name: \(name)"])
        }
        return encryptedName
    }

    func uploadFile(repo: Repo, encryptedParentPath: String, name: String, content: String) async
        -> RepoFile
    {
        await uploadFile(
            repo: repo, encryptedParentPath: encryptedParentPath, name: name,
            data: content.data(using: .utf8)!)
    }

    func uploadFile(repo: Repo, encryptedParentPath: String, name: String, data: Data) async
        -> RepoFile
    {
        let browserId = mobileVault.repoFilesBrowsersCreate(
            source: .storage(repoId: repo.id, encryptedPath: encryptedParentPath),
            options: RepoFilesBrowserOptions(selectName: nil))

        defer {
            mobileVault.repoFilesBrowsersDestroy(browserId: browserId)
        }

        mobileVault.transfersUploadBytes(
            repoId: repo.id, encryptedParentPath: encryptedParentPath, name: name,
            bytes: data)

        return await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesBrowsersInfoSubscribe(browserId: browserId, cb: cb)
            },
            getData: { v, id in
                if let info = v.repoFilesBrowsersInfoData(id: id) {
                    return info.items.first(where: { $0.file.name == name }).map { $0.file }
                }
                return nil
            }
        )
    }

    func getFileContent(repo: Repo, encryptedPath: String) async
        -> String
    {
        let detailsId = mobileVault.repoFilesDetailsCreate(
            repoId: repo.id, encryptedPath: encryptedPath, isEditing: false,
            options: RepoFilesDetailsOptions(
                loadContent: FilesFilter(categories: [.code, .text], exts: []),
                autosaveIntervalMs: 20000))

        defer {
            mobileVault.repoFilesDetailsDestroy(detailsId: detailsId)
        }

        return await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsContentBytesSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                if let data = v.repoFilesDetailsContentBytesData(id: id) {
                    if let dataString = String(data: data, encoding: .utf8) {
                        return dataString
                    }
                }
                return nil
            }
        )
    }

    func waitForFileContent(repo: Repo, encryptedPath: String, expectedContent: String) async {
        let detailsId = mobileVault.repoFilesDetailsCreate(
            repoId: repo.id, encryptedPath: encryptedPath, isEditing: false,
            options: RepoFilesDetailsOptions(
                loadContent: FilesFilter(categories: [.code, .text], exts: []),
                autosaveIntervalMs: 20000))

        defer {
            mobileVault.repoFilesDetailsDestroy(detailsId: detailsId)
        }

        let _ = await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsContentBytesSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                if let data = v.repoFilesDetailsContentBytesData(id: id) {
                    if let dataString = String(data: data, encoding: .utf8) {
                        if dataString == expectedContent {
                            return ()
                        }
                    }
                }
                return nil
            }
        )
    }

    func setFileContent(repo: Repo, encryptedPath: String, content: String) async {
        let detailsId = mobileVault.repoFilesDetailsCreate(
            repoId: repo.id, encryptedPath: encryptedPath, isEditing: true,
            options: RepoFilesDetailsOptions(
                loadContent: FilesFilter(categories: [.code, .text], exts: []),
                autosaveIntervalMs: 20000))

        defer {
            mobileVault.repoFilesDetailsDestroy(detailsId: detailsId)
        }

        // wait for content loaded
        let _ = await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsInfoSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                if let data = v.repoFilesDetailsInfoData(id: id) {
                    if case .loaded = data.contentStatus {
                        return ()
                    }
                }
                return nil
            }
        )

        mobileVault.repoFilesDetailsSetContent(
            detailsId: detailsId, content: content.data(using: .utf8)!)

        mobileVault.repoFilesDetailsSave(detailsId: detailsId)

        // wait for saved
        let _ = await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsInfoSubscribe(detailsId: detailsId, cb: cb)
            },
            getData: { v, id in
                if let data = v.repoFilesDetailsInfoData(id: id) {
                    if !data.isDirty {
                        return ()
                    }
                }
                return nil
            }
        )
    }

    func ensureFile(repo: Repo, encryptedPath: String) async throws {
        let (encryptedParentPath, _) = try splitParentName(encryptedPath)

        let browserId = mobileVault.repoFilesBrowsersCreate(
            source: .storage(repoId: repo.id, encryptedPath: encryptedParentPath),
            options: .init(selectName: nil))

        defer {
            mobileVault.repoFilesBrowsersDestroy(browserId: browserId)
        }

        return await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.repoFilesBrowsersInfoSubscribe(browserId: browserId, cb: cb)
            },
            getData: { v, id in
                if let data = v.repoFilesBrowsersInfoData(id: id) {
                    if case .loaded = data.status {
                        return ()
                    }
                }
                return nil
            }
        )
    }

    func renameFile(
        repo: Repo,
        encryptedPath: String,
        newName: String
    ) async throws {
        try await ensureFile(repo: repo, encryptedPath: encryptedPath)

        mobileVault.repoFilesRenameFile(
            repoId: repo.id,
            encryptedPath: encryptedPath
        )

        await promptDialogFill(value: newName)
    }

    func deleteFile(
        repo: Repo,
        encryptedPath: String
    ) async throws {
        try await ensureFile(repo: repo, encryptedPath: encryptedPath)

        mobileVault.repoFilesDeleteFile(
            repoId: repo.id,
            encryptedPath: encryptedPath
        )

        await confirmDialog()
    }

    func promptDialogFill(value: String) async {
        await confirmDialog(inputValue: value)
    }

    func confirmDialog(inputValue: String? = nil) async {
        // get first dialog id
        let dialogId = await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.dialogsSubscribe(cb: cb)
            },
            getData: { v, id in
                if let dialogs = v.dialogsData(id: id) {
                    if !dialogs.isEmpty {
                        return dialogs[0]
                    }
                }
                return nil
            }
        )

        // wait for the dialog to exist
        let _ = await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.dialogsDialogSubscribe(dialogId: dialogId, cb: cb)
            },
            getData: { v, id in
                v.dialogsDialogData(id: id)
            }
        )

        // if input value is set, set the value
        if let value = inputValue {
            mobileVault.dialogsSetInputValue(
                dialogId: dialogId,
                value: value
            )
        }

        // confirm the dialog
        mobileVault.dialogsConfirm(dialogId: dialogId)

        // wait for the dialog to be destroyed
        await subscriptionWait(
            mobileVault: mobileVault,
            subscribe: { v, cb in
                v.dialogsDialogSubscribe(dialogId: dialogId, cb: cb)
            },
            getData: { v, id in
                if v.dialogsDialogData(id: id) == nil {
                    return ()
                }
                return nil
            }
        )
    }

}
