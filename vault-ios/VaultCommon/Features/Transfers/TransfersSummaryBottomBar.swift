import SwiftUI
import VaultMobile

public struct TransfersSummaryBottomBar: View {
    public let summary: TransfersSummary

    public var body: some View {
        VStack {
            HStack {
                VStack(alignment: .leading) {
                    Text(
                        LocalizedStringResource(
                            "ios.transfers.summary.transfers_done",
                            defaultValue: "\(summary.doneCount) / \(summary.totalCount) done",
                            bundle: #bundle,
                            comment:
                                "Summary text in transfers footer showing completed versus total transfer count."
                        )
                    )
                    .padding(.bottom, 2)
                    Text(summary.sizeProgressDisplay)
                }

                Spacer()

                if summary.isTransferring {
                    VStack(alignment: .trailing) {
                        Text(summary.speedDisplay).padding(.bottom, 2)
                        Text(
                            LocalizedStringResource(
                                "ios.transfers.summary.time_remaining",
                                defaultValue: "\(summary.remainingTimeDisplay) remaining",
                                bundle: #bundle,
                                comment:
                                    "Summary text in transfers footer showing estimated time remaining."
                            )
                        )
                        .multilineTextAlignment(.trailing)
                    }
                }
            }
            .padding(.bottom, 5)

            ProgressView(value: Double(summary.percentage), total: 100)
        }
    }
}

public struct TransfersSummaryBottomBar_Previews: PreviewProvider {
    static public var previews: some View {
        TransfersSummaryBottomBar(summary: PreviewsData.transfersSummary).previewLayout(
            .fixed(width: 300, height: 150))
    }
}
