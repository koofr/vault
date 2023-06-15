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

    private var transfersIsActive: Subscription<Bool>
    private var transfersWasActive: Bool
    private var transfersAborted: Bool

    public init(container: Container, onDismiss: @escaping () -> Void) {
        self.container = container
        self.onDismiss = onDismiss

        self.state = .preparingFiles

        self.transfersWasActive = false

        self.transfersAborted = false

        self.transfersIsActive = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersIsActiveSubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersIsActiveData(id: id)
            })

        self.transfersIsActive.setOnData { [weak self] isActive in
            if let self = self {
                if let isActive = isActive {
                    if isActive && !self.transfersWasActive {
                        self.transfersWasActive = true
                    }
                    if !isActive && self.transfersWasActive {
                        if transfersAborted {
                            dismiss()
                        } else {
                            self.state = .done
                        }
                    }
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
