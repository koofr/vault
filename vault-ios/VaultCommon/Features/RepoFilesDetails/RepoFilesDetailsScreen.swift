import AVFoundation
import AVKit
import SwiftUI
import VaultMobile

public struct RepoFilesDetailsScreen: View {
    @ObservedObject var vm: RepoFilesDetailsScreenViewModel

    @ObservedObject private var info: Subscription<RepoFilesDetailsInfo>

    @State private var shareViewPresented = false

    public init(vm: RepoFilesDetailsScreenViewModel) {
        self.vm = vm

        self.info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsInfoSubscribe(detailsId: vm.detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsInfoData(id: id)
            })
    }

    public var body: some View {
        HStack {
            RepoFilesDetailsContent(vm: vm, info: info)
        }
        .navigationTitle(info.data?.fileName ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                TransfersButton(container: vm.container)
            }

            ToolbarItem(placement: .primaryAction) {
                switch vm.content {
                case .downloaded(let localFileURL, _):
                    Button(
                        action: {
                            shareViewPresented.toggle()
                        },
                        label: {
                            Image(systemName: "square.and.arrow.up")
                        }
                    )
                    .sheet(isPresented: $shareViewPresented) {
                        ActivityView(activityItems: [localFileURL], showOpenInDownloads: false)
                    }
                default:
                    EmptyView()
                }
            }
        }
    }
}

public struct RepoFilesDetailsContent: View {
    @ObservedObject private var vm: RepoFilesDetailsScreenViewModel
    @ObservedObject private var info: Subscription<RepoFilesDetailsInfo>

    public init(vm: RepoFilesDetailsScreenViewModel, info: Subscription<RepoFilesDetailsInfo>) {
        self.vm = vm
        self.info = info
    }

    public var body: some View {
        switch vm.content {
        case .loading:
            ProgressView()
        case .downloading:
            VStack {
                if let info = info.data {
                    if let transferId = info.transferId {
                        RepoFilesDetailsContentDownloadingTransfer(vm: vm, transferId: transferId)
                    } else if let error = info.error {
                        Text("Error: \(error)")
                    } else {
                        LoadingView()
                    }
                } else {
                    LoadingView()
                }
            }
        case .downloaded(_, let data):
            switch data {
            case .image(let image):
                ZoomableScrollView {
                    Image(uiImage: image)
                }
            case .gifImage(let image):
                RawImage(image: image)
            case .media(let player):
                ZStack {
                    Color.black.ignoresSafeArea(.container, edges: [.leading, .trailing, .bottom])

                    VideoPlayer(player: player)
                }
            case .webViewAsset(let asset):
                WebView(asset: asset)
                    .ignoresSafeArea(.container, edges: [.bottom])
            case .error(let error):
                Text("Error: \(error)")
            }
        case .notSupported(let file):
            RepoFilesDetailsContentNotSupported(vm: vm, file: file)
        }
    }
}

public struct RepoFilesDetailsContentDownloadingTransfer: View {
    private let vm: RepoFilesDetailsScreenViewModel

    @ObservedObject private var transfer: Subscription<Transfer>

    public init(vm: RepoFilesDetailsScreenViewModel, transferId: UInt32) {
        self.vm = vm

        self.transfer = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.transfersTransferSubscribe(transferId: transferId, cb: cb)
            },
            getData: { v, id in
                v.transfersTransferData(id: id)
            })
    }

    public var body: some View {
        if let transfer = transfer.data {
            TransferInfoView(
                transfer: transfer,
                onRetry: {
                    vm.container.mobileVault.transfersRetry(id: transfer.id)
                })
        }
    }
}

public struct RepoFilesDetailsContentNotSupported: View {
    public let vm: RepoFilesDetailsScreenViewModel
    public let file: RepoFile

    public init(vm: RepoFilesDetailsScreenViewModel, file: RepoFile) {
        self.vm = vm
        self.file = file
    }

    public var body: some View {
        VStack {
            Text("Not supported").padding()

            Button {
                vm.container.downloadHelper.downloadRepoFile(file: file)
            } label: {
                Text("Download")
            }
            .padding()
        }
    }
}
