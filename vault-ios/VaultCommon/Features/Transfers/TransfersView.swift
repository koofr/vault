import SwiftUI
import VaultMobile

public struct TransfersView: View {
    public let container: Container
    public let onAbort: (() -> Void)?

    @ObservedObject private var summary: Subscription<TransfersSummary>
    @ObservedObject private var transfers: Subscription<[Transfer]>

    public init(container: Container, onAbort: (() -> Void)? = nil) {
        self.container = container
        self.onAbort = onAbort

        summary = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersSummarySubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersSummaryData(id: id)
            })

        transfers = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.transfersListSubscribe(cb: cb)
            },
            getData: { v, id in
                v.transfersListData(id: id)
            })
    }

    public var body: some View {
        VStack(spacing: 0) {
            List(transfers.data!, id: \.id) { transfer in
                TransferRow(
                    container: container,
                    transfer: transfer,
                    onRetry: {
                        container.mobileVault.transfersRetry(id: transfer.id)
                    },
                    onAbort: {
                        onAbort?()

                        container.mobileVault.transfersAbort(id: transfer.id)
                    },
                    onOpen: {
                        container.mobileVault.transfersOpen(id: transfer.id)
                    })
            }

            Spacer(minLength: 0)

            if let summary = summary.data {
                VStack(spacing: 0) {
                    VStack {
                        TransfersSummaryBottomBar(summary: summary)
                    }
                    .padding(15)
                }
            }
        }
        .toolbar {
            if let summary = summary.data {
                if summary.canRetryAll {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button {
                            container.mobileVault.transfersRetryAll()
                        } label: {
                            Text(
                                LocalizedStringResource(
                                    "ios.transfers.retry_all.button",
                                    defaultValue: "Retry all",
                                    bundle: #bundle,
                                    comment:
                                        "Toolbar button in transfers view that retries all failed transfers."
                                )
                            )
                        }
                    }
                }
            }

            if let summary = summary.data {
                if summary.canAbortAll {
                    if summary.isAllDone {
                        ToolbarItem(placement: .navigationBarTrailing) {
                            Button {
                                onAbort?()

                                container.mobileVault.transfersAbortAll()
                            } label: {
                                Text(
                                    LocalizedStringResource(
                                        "ios.transfers.clear.button",
                                        defaultValue: "Clear",
                                        bundle: #bundle,
                                        comment:
                                            "Toolbar button in transfers view that clears completed transfer history."
                                    )
                                )
                            }
                        }
                    } else {
                        ToolbarItem(placement: .destructiveAction) {
                            Button(role: .destructive) {
                                onAbort?()

                                container.mobileVault.transfersAbortAll()
                            } label: {
                                Text(
                                    LocalizedStringResource(
                                        "ios.transfers.cancel_all.button",
                                        defaultValue: "Cancel all",
                                        bundle: #bundle,
                                        comment:
                                            "Destructive toolbar button in transfers view that aborts all active transfers."
                                    )
                                )
                                .foregroundColor(Color(.systemRed))
                            }
                        }
                    }
                }
            }
        }
    }
}
