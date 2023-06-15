import SwiftUI
import VaultMobile

public struct TransfersSummaryBottomBar: View {
    public let summary: TransfersSummary

    public var body: some View {
        VStack {
            HStack {
                VStack(alignment: .leading) {
                    Text("\(summary.doneCount) / \(summary.totalCount) done").padding(.bottom, 2)
                    Text(summary.sizeProgressDisplay)
                }

                Spacer()

                if summary.isTransferring {
                    VStack(alignment: .trailing) {
                        Text(summary.speedDisplay).padding(.bottom, 2)
                        Text("\(summary.remainingTimeDisplay) remaining").multilineTextAlignment(
                            .trailing)
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
