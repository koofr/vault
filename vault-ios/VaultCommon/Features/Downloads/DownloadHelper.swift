import Foundation
import VaultMobile

public class DownloadHelper {
    private let mobileVault: MobileVault
    private let sheets: Sheets
    private let transfersSheetController: TransfersSheetController
    private let storageHelper: StorageHelper

    public init(
        mobileVault: MobileVault, sheets: Sheets,
        transfersSheetController: TransfersSheetController, storageHelper: StorageHelper
    ) {
        self.mobileVault = mobileVault
        self.sheets = sheets
        self.transfersSheetController = transfersSheetController
        self.storageHelper = storageHelper
    }

    public func downloadRepoFile(file: RepoFile) {
        do {
            let localFilePath = try storageHelper.getDownloadsDir().path(percentEncoded: false)

            mobileVault.transfersDownloadFile(
                repoId: file.repoId,
                encryptedPath: file.encryptedPath,
                localFilePath: localFilePath,
                appendName: true,
                autorename: true,
                onOpen: TransfersDownloadOpenFn { [weak self] localFilePath in
                    if let self = self {
                        self.onOpen(localFilePath: localFilePath)
                    }
                },
                onDone: TransfersDownloadDoneFn { _ in }
            )

            transfersSheetController.showWhenActive()
        } catch {
            print("DownloadHelper downloadRepoFile error: \(error)")
        }
    }

    public func downloadRepoFilesBrowsersSelected(browserId: UInt32) {
        do {
            let localFilePath = try storageHelper.getDownloadsDir().path(percentEncoded: false)

            mobileVault.repoFilesBrowsersDownloadSelectedFile(
                browserId: browserId,
                localFilePath: localFilePath,
                appendName: true,
                autorename: true,
                onOpen: TransfersDownloadOpenFn { [weak self] localFilePath in
                    if let self = self {
                        self.onOpen(localFilePath: localFilePath)
                    }
                },
                onDone: TransfersDownloadDoneFn { _ in }
            )

            transfersSheetController.showWhenActive()
        } catch {
            print("DownloadHelper downloadRepoFilesBrowsersSelected error: \(error)")
        }
    }

    func onOpen(localFilePath: String) {
        let localFileURL = URL(fileURLWithPath: localFilePath)

        sheets.show { _, _ in
            ActivityView(activityItems: [localFileURL], showOpenInDownloads: true)
        }
    }
}
