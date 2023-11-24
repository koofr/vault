import Foundation
import VaultMobile

public struct PreviewsData {
    static let repos: [Repo] = [
        Repo(
            id: "49c1953f-efb0-4b5a-bf06-02da11e2a9d6",
            name: "Vault",
            mountId: "7ac98cb0-93d6-433a-878b-fee805e9d68f",
            path: "/Vault",
            state: RepoState.unlocked,
            added: 1_666_300_393_650,
            webUrl:
                "https://app.koofr.net/app/storage/7ac98cb0-93d6-433a-878b-fee805e9d68f?path=%2FVault"
        ),
        Repo(
            id: "ab72e550-64cf-4eaf-9f4c-4dda6459a045",
            name: "Vault 1",
            mountId: "7ac98cb0-93d6-433a-878b-fee805e9d68f",
            path: "/Vault 1",
            state: RepoState.locked,
            added: 1_666_300_428_883,
            webUrl:
                "https://app.koofr.net/app/storage/7ac98cb0-93d6-433a-878b-fee805e9d68f?path=%2FVault%201"
        ),
    ]

    static let repoConfig = RepoConfig(
        name: "My safe box",
        location: RemoteFilesLocation(
            mountId: "c1450ad8-1f79-4f2f-938d-0a4bfd927313", path: "/My safe box"),
        password: "password", salt: "salt",
        rcloneConfig:
            "[my-safe-box]\ntype=crypt\nremote=koofr:/My safe box\npassword=QRnSVJm14OpUhFyUOMjlelemU14sXMIV\npassword2=ntSsq_groe8-AMAKUEjGbapCTQU"
    )

    static let transfersSummary = TransfersSummary(
        totalCount: 2,
        doneCount: 0,
        failedCount: 0,
        sizeProgressDisplay: "14.6 / 219.4 MB",
        percentage: 6,
        remainingTimeDisplay: "2m 19s",
        speedDisplay: "1.5 MB/s",
        isTransferring: true,
        isAllDone: false,
        canRetryAll: false,
        canAbortAll: true
    )

    static let transfersList: [Transfer] = [
        Transfer(
            id: 0,
            typ: .upload,
            name: "IMG_1401.mp4",
            fileIconAttrs: FileIconAttrs(
                category: .video,
                isDl: false,
                isUl: false,
                isDownloadTransfer: false,
                isUploadTransfer: true,
                isExport: false,
                isImport: false,
                isAndroid: false,
                isIos: false,
                isVaultRepo: false,
                isError: false
            ),
            size: 134_562_934,
            sizeDisplay: "128.3 MB",
            sizeProgressDisplay: "4.8 / 128.3 MB",
            percentage: 4,
            transferredBytes: 5_046_272,
            transferredDisplay: "4.8 MB",
            speedDisplay: "495.1 KB/s",
            state: .transferring,
            canRetry: false,
            canOpen: false
        ),
        Transfer(
            id: 1,
            typ: .download,
            name: "Report 2023.pdf",
            fileIconAttrs: FileIconAttrs(
                category: .pdf,
                isDl: false,
                isUl: false,
                isDownloadTransfer: true,
                isUploadTransfer: false,
                isExport: false,
                isImport: false,
                isAndroid: false,
                isIos: false,
                isVaultRepo: false,
                isError: false
            ),
            size: 95_514_160,
            sizeDisplay: "91.1 MB",
            sizeProgressDisplay: "9.9 / 91.1 MB",
            percentage: 11,
            transferredBytes: 10_354_688,
            transferredDisplay: "9.9 MB",
            speedDisplay: "1.3 MB/s",
            state: .transferring,
            canRetry: false,
            canOpen: false
        ),
        Transfer(
            id: 2,
            typ: .upload,
            name: "Documents.zip",
            fileIconAttrs: FileIconAttrs(
                category: .archive,
                isDl: false,
                isUl: false,
                isDownloadTransfer: false,
                isUploadTransfer: true,
                isExport: false,
                isImport: false,
                isAndroid: false,
                isIos: false,
                isVaultRepo: false,
                isError: false
            ),
            size: 10_485_760,
            sizeDisplay: "10 MB",
            sizeProgressDisplay: "0 / 10 MB",
            percentage: 0,
            transferredBytes: 0,
            transferredDisplay: "0 MB",
            speedDisplay: "0 MB/s",
            state: .failed(
                error:
                    "Unknown error: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec pretium, tortor sit amet condimentum tempus, diam nulla feugiat purus, a facilisis mi nulla at felis. Donec mattis a metus id porta."
            ),
            canRetry: true,
            canOpen: false
        ),
        Transfer(
            id: 3,
            typ: .download,
            name: "Photo.jpg",
            fileIconAttrs: FileIconAttrs(
                category: .image,
                isDl: false,
                isUl: false,
                isDownloadTransfer: true,
                isUploadTransfer: false,
                isExport: false,
                isImport: false,
                isAndroid: false,
                isIos: false,
                isVaultRepo: false,
                isError: false
            ),
            size: 1_354_936,
            sizeDisplay: "1.29 MB",
            sizeProgressDisplay: "1.29 / 1.29 MB",
            percentage: 100,
            transferredBytes: 1_354_936,
            transferredDisplay: "1.29 MB",
            speedDisplay: nil,
            state: .done,
            canRetry: false,
            canOpen: true
        ),
    ]
}
