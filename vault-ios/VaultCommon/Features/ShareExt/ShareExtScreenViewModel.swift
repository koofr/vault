import Foundation
import VaultMobile

public enum ShareExtState {
    case preparingFiles
    case noFiles
    case shareTarget(vm: ShareTargetViewModel)
    case transfers
    case done
}

public class ShareExtScreenViewModel: ObservableObject {
    public let container: Container
    public let onDismiss: () -> Void

    @Published public var state: ShareExtState

    private var transfersDoneSessionsCount: Subscription<UInt32>
    private var transfersDoneSessionsCountBeforeUpload: UInt32?
    private var transfersAborted: Bool

    public init(container: Container, onDismiss: @escaping () -> Void) {
        self.container = container
        self.onDismiss = onDismiss

        self.state = .preparingFiles

        self.transfersAborted = false

        self.transfersDoneSessionsCount = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersDoneSessionsCountSubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersDoneSessionsCountData(id: id)
            })

        self.transfersDoneSessionsCount.setOnData { [weak self] transfersDoneSessionsCount in
            // If there are more done sessions then before the upload, transfers
            // are done.
            if let self = self,
                let transfersDoneSessionsCount = transfersDoneSessionsCount,
                let transfersDoneSessionsCountBeforeUpload = self
                    .transfersDoneSessionsCountBeforeUpload,
                transfersDoneSessionsCount > transfersDoneSessionsCountBeforeUpload
            {
                if transfersAborted {
                    dismiss()
                } else {
                    self.state = .done
                }
            }
        }
    }

    @MainActor
    public func handleItemProviders(_ itemProviders: [NSItemProvider]) {
        Task {
            let files = await container.uploadHelper.itemProvidersToFiles(
                itemProviders: itemProviders, loadFileRepresentation: false)

            handleFiles(files)
        }
    }

    private func handleFiles(_ files: [UploadFile]) {
        if files.isEmpty {
            state = .noFiles
            return
        }

        let vm = ShareTargetViewModel(
            container: container,
            files: files,
            beforeUpload: {
                self.transfersDoneSessionsCountBeforeUpload = self.transfersDoneSessionsCount.data
            },
            onUpload: {
                self.state = .transfers
            },
            onCancel: {
                self.dismiss()
            }
        )

        state = .shareTarget(vm: vm)
    }

    func onTransfersAbort() {
        transfersAborted = true
    }

    func dismiss() {
        onDismiss()
    }
}
