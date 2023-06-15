import SwiftUI
import VaultMobile

public struct TransferInfoView: View {
    public let transfer: Transfer
    public let onRetry: () -> Void

    init(transfer: Transfer, onRetry: @escaping () -> Void) {
        self.transfer = transfer
        self.onRetry = onRetry
    }

    public var body: some View {
        VStack {
            if let percentage = transfer.percentage {
                ProgressView(value: Double(percentage), total: 100)
                    .padding(.bottom, 15)
            } else {
                ProgressView()
                    .padding(.bottom, 15)
            }

            Text(getTransferDescription(state: transfer.state))
                .multilineTextAlignment(.center)
                .padding(.bottom, 5)

            if let sizeProgressDisplay = transfer.sizeProgressDisplay {
                Text(sizeProgressDisplay)
            }

            if transfer.canRetry {
                Button {
                    onRetry()
                } label: {
                    Text("Try again")
                }
                .padding(.top, 15)
            }
        }
        .padding()
    }
}

struct TransferInfoView_Previews: PreviewProvider {
    static var previews: some View {
        TransferInfoView(
            transfer: PreviewsData.transfersList[0],
            onRetry: {}
        )
        .previewDisplayName("Video - Transferring")
        TransferInfoView(
            transfer: PreviewsData.transfersList[2],
            onRetry: {}
        )
        .previewDisplayName("ZIP - Failed")
    }
}
