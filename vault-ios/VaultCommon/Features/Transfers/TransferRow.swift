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
                        Text("Open").foregroundColor(Color(.link))
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
                        Text("Retry").foregroundColor(Color(.link))
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
                        Text("Cancel").foregroundColor(Color(.systemRed))
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

public func getTransferDescription(state: TransferState) -> String {
    switch state {
    case .waiting:
        return "Waiting"
    case .processing:
        return "Processing"
    case .transferring:
        return "Transferring"
    case .failed(let err):
        return "Failed: \(err)"
    case .done:
        return "Done"
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
