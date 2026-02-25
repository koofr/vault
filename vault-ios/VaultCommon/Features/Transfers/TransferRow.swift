import SwiftUI
import VaultMobile

public struct TransferRow: View {
    public let container: Container
    public let transfer: Transfer
    public let onRetry: () -> Void
    public let onAbort: () -> Void
    public let onOpen: () -> Void

    public var body: some View {
        HStack {
            FileIcon(
                fileIconCache: container.fileIconCache, attrs: transfer.fileIconAttrs
            )
            .padding(.trailing, 7)
            VStack(alignment: .leading, spacing: 0) {
                Text(transfer.name)
                    .frame(height: 30)
                    .truncationMode(.middle)
                    .font(.system(.body))
                    .foregroundColor(Color(.label))

                Text(getTransferDescription(state: transfer.state))
                    .font(.system(.footnote))
                    .foregroundColor(Color(.secondaryLabel))
            }
            Spacer()
            // button actions must be empty otherwise click on the row triggers
            // the action
            if transfer.canOpen {
                Button(
                    action: {},
                    label: {
                        Text(
                            LocalizedStringResource(
                                "ios.transfers.transfer.open.button",
                                defaultValue: "Open",
                                bundle: #bundle,
                                comment:
                                    "Action button on a transfer row to open the transferred file after completion."
                            )
                        )
                        .foregroundColor(Color(.link))
                    }
                )
                .onTapGesture {
                    onOpen()
                }
            }
            if transfer.canRetry {
                Button(
                    action: {},
                    label: {
                        Text(
                            LocalizedStringResource(
                                "ios.transfers.transfer.retry.button",
                                defaultValue: "Retry",
                                bundle: #bundle,
                                comment:
                                    "Action button on a transfer row to retry a failed transfer."
                            )
                        )
                        .foregroundColor(Color(.link))
                    }
                )
                .onTapGesture {
                    onRetry()
                }
            }
            switch transfer.state {
            case .done:
                Button(
                    action: {},
                    label: {
                        Image(systemName: "xmark").tint(Color(.systemGray))
                    }
                )
                .onTapGesture {
                    onAbort()
                }
            default:
                Button(
                    role: .destructive, action: {},
                    label: {
                        Text(
                            LocalizedStringResource(
                                "ios.transfers.transfer.cancel.button",
                                defaultValue: "Cancel",
                                bundle: #bundle,
                                comment:
                                    "Action button on a transfer row to cancel an active transfer."
                            )
                        )
                        .foregroundColor(Color(.systemRed))
                    }
                )
                .onTapGesture {
                    onAbort()
                }
            }
        }
        .frame(minHeight: 45)
    }
}

public func getTransferDescription(state: TransferState) -> LocalizedStringResource {
    switch state {
    case .waiting:
        return LocalizedStringResource(
            "ios.transfers.transfer.state.waiting",
            defaultValue: "Waiting",
            bundle: #bundle,
            comment: "Transfer status text shown when a transfer is queued and waiting."
        )
    case .processing:
        return LocalizedStringResource(
            "ios.transfers.transfer.state.processing",
            defaultValue: "Processing",
            bundle: #bundle,
            comment: "Transfer status text shown while preparing transfer."
        )
    case .transferring:
        return LocalizedStringResource(
            "ios.transfers.transfer.state.transferring",
            defaultValue: "Transferring",
            bundle: #bundle,
            comment: "Transfer status text shown while file is actively transferring."
        )
    case .failed(let err):
        return LocalizedStringResource(
            "ios.transfers.transfer.state.failed",
            defaultValue: "Failed: \(err)",
            bundle: #bundle,
            comment: "Transfer status text shown when a transfer fails; includes error details."
        )
    case .done:
        return LocalizedStringResource(
            "ios.transfers.transfer.state.done",
            defaultValue: "Done",
            bundle: #bundle,
            comment: "Transfer status text shown when a transfer finishes successfully."
        )
    }
}

public struct TransferRow_Previews: PreviewProvider {
    static public var previews: some View {
        Group {
            TransferRow(
                container: Container(),
                transfer: PreviewsData.transfersList[0],
                onRetry: {},
                onAbort: {},
                onOpen: {}
            ).previewDisplayName("Video - Transferring")
            TransferRow(
                container: Container(),
                transfer: PreviewsData.transfersList[1],
                onRetry: {},
                onAbort: {},
                onOpen: {}
            ).previewDisplayName("PDF - Transferring")
            TransferRow(
                container: Container(),
                transfer: PreviewsData.transfersList[2],
                onRetry: {},
                onAbort: {},
                onOpen: {}
            ).previewDisplayName("ZIP - Failed")
            TransferRow(
                container: Container(),
                transfer: PreviewsData.transfersList[3],
                onRetry: {},
                onAbort: {},
                onOpen: {}
            ).previewDisplayName("JPG - Done")
        }.previewLayout(.fixed(width: 300, height: 200))
    }
}
