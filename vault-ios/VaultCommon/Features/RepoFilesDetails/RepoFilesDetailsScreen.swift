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

        self.info = vm.info
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

            if let info = info.data {
                if info.isEditing {
                    ToolbarItem(placement: .primaryAction) {
                        Button(
                            action: {
                                vm.container.mobileVault.repoFilesDetailsSave(
                                    detailsId: vm.detailsId)
                            },
                            label: {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_files_details.save.button",
                                        defaultValue: "Save",
                                        bundle: #bundle,
                                        comment:
                                            "Toolbar button in file details editor mode that saves pending text changes."
                                    )
                                )
                            }
                        )
                        .disabled(!info.isDirty)
                    }

                    ToolbarItem(placement: .confirmationAction) {
                        Button(
                            action: {
                                vm.container.mobileVault.repoFilesDetailsEditCancel(
                                    detailsId: vm.detailsId)
                            },
                            label: {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_files_details.done.button",
                                        defaultValue: "Done",
                                        bundle: #bundle,
                                        comment:
                                            "Toolbar button in file details editor mode that exits editing."
                                    )
                                )
                            }
                        )
                    }
                } else {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Menu {
                            if case .textEditor(_) = vm.content {
                                Button(
                                    action: {
                                        vm.container.mobileVault.repoFilesDetailsEdit(
                                            detailsId: vm.detailsId)
                                    },
                                    label: {
                                        Label {
                                            Text(
                                                LocalizedStringResource(
                                                    "ios.repo_files_details.edit.menu_item",
                                                    defaultValue: "Edit",
                                                    bundle: #bundle,
                                                    comment:
                                                        "Overflow menu item in file details for entering edit mode on text files."
                                                )
                                            )
                                        } icon: {
                                            Image(systemName: "square.and.pencil")
                                        }
                                    }
                                )
                            }

                            switch vm.content {
                            case .downloaded(_, _), .textEditor(_):
                                Button(
                                    action: {
                                        shareViewPresented.toggle()
                                    },
                                    label: {
                                        Label {
                                            Text(
                                                LocalizedStringResource(
                                                    "ios.repo_files_details.share.menu_item",
                                                    defaultValue: "Share",
                                                    bundle: #bundle,
                                                    comment:
                                                        "Overflow menu item in file details for opening the system share sheet."
                                                )
                                            )
                                        } icon: {
                                            Image(systemName: "square.and.arrow.up")
                                        }
                                    }
                                )
                            default:
                                EmptyView()
                            }

                            if let repoId = info.repoId {
                                if let encryptedPath = info.encryptedPath {
                                    Button(
                                        action: {
                                            vm.container.mobileVault.repoFilesRenameFile(
                                                repoId: repoId, encryptedPath: encryptedPath)
                                        },
                                        label: {
                                            Label {
                                                Text(
                                                    LocalizedStringResource(
                                                        "ios.repo_files_details.rename.menu_item",
                                                        defaultValue: "Rename",
                                                        bundle: #bundle,
                                                        comment:
                                                            "Overflow menu item in file details for renaming the current file."
                                                    )
                                                )
                                            } icon: {
                                                Image(systemName: "pencil")
                                            }
                                        }
                                    )
                                }
                            }

                            Button(role: .destructive) {
                                vm.container.mobileVault.repoFilesDetailsDelete(
                                    detailsId: vm.detailsId)
                            } label: {
                                Label {
                                    Text(
                                        LocalizedStringResource(
                                            "ios.repo_files_details.delete.menu_item",
                                            defaultValue: "Delete",
                                            bundle: #bundle,
                                            comment:
                                                "Destructive overflow menu item in file details for deleting the current file."
                                        )
                                    )
                                } icon: {
                                    Image(systemName: "trash")
                                }
                            }
                        } label: {
                            Image(systemName: "ellipsis.circle")
                        }
                        .sheet(isPresented: $shareViewPresented) {
                            switch vm.content {
                            case .downloaded(let localFileURL, _):
                                ActivityView(
                                    activityItems: [localFileURL], showOpenInDownloads: false)
                            case .textEditor(_):
                                RepoFilesDetailsTextEditorActivityView(vm: vm)
                            default:
                                EmptyView()
                            }
                        }
                    }
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
                        ErrorView(
                            errorText: error,
                            onRetry: {
                                vm.retryLoad()
                            })
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
                ErrorView(errorText: error)
            }
        case .textEditor:
            VStack {
                if let info = info.data {
                    if info.isEditing {
                        RepoFilesDetailsEditorInfo(vm: vm, info: info)

                        Divider()
                    }
                }

                RepoFilesDetailsTextEditor(vm: vm)
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
            Text(
                LocalizedStringResource(
                    "ios.repo_files_details.not_supported.label",
                    defaultValue: "Not supported",
                    bundle: #bundle,
                    comment:
                        "Message shown in file details when in-app preview/edit is not supported for the file type."
                )
            )
            .padding()

            Button {
                vm.container.downloadHelper.downloadRepoFile(file: file)
            } label: {
                Text(
                    LocalizedStringResource(
                        "ios.repo_files_details.download.button",
                        defaultValue: "Download",
                        bundle: #bundle,
                        comment:
                            "Button in unsupported file details view that downloads the file locally."
                    )
                )
            }
            .padding()
        }
    }
}
