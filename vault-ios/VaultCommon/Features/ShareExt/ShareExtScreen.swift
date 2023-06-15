import SwiftUI
import VaultMobile

public struct ShareExtScreen: View {
    @ObservedObject var vm: ShareExtScreenViewModel

    @ObservedObject private var oauth2Status: Subscription<Status>

    public init(vm: ShareExtScreenViewModel) {
        self.vm = vm

        self.oauth2Status = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.oauth2StatusSubscribe(cb: cb)
            },
            getData: { v, id in
                v.oauth2StatusData(id: id)
            })
    }

    public var body: some View {
        ZStack {
            if let oauth2Status = oauth2Status.data {
                switch oauth2Status {
                case .initial:
                    ShareExtScreenUnauthenticated(vm: vm)
                case .loading(_):
                    ShareExtScreenLoading(vm: vm)
                case .loaded:
                    switch vm.state {
                    case .preparingFiles:
                        ShareExtScreenPreparingFiles(vm: vm)
                    case .noFiles:
                        ShareExtScreenNoFiles(vm: vm)
                    case .shareTarget(let vm):
                        ShareTargetNavigation(vm: vm)
                    case .transfers:
                        NavigationView {
                            TransfersView(container: vm.container, onAbort: vm.onTransfersAbort)
                        }
                        .navigationViewStyle(.stack)
                    case .done:
                        ShareExtScreenDone(vm: vm)
                    }
                case .err(let error, _):
                    ErrorView(
                        errorText: error,
                        onRetry: {
                            vm.container.mobileVault.load()
                        })
                }
            }

            Overlays(container: vm.container)
        }
    }
}

struct ShareExtScreenUnauthenticated: View {
    var vm: ShareExtScreenViewModel

    init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    public var body: some View {
        NavigationView {
            VStack {
                Text("Not signed in").font(.largeTitle).padding(.bottom, 10)

                Text("Open Koofr Vault app and sign in.")
            }
            .navigationBarTitle("", displayMode: .inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        vm.dismiss()
                    } label: {
                        Text("Dismiss").bold()
                    }
                }
            }
        }
        .navigationViewStyle(.stack)
    }
}

struct ShareExtScreenLoading: View {
    var vm: ShareExtScreenViewModel

    init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    public var body: some View {
        NavigationView {
            LoadingView()
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            vm.dismiss()
                        } label: {
                            Text("Cancel").bold()
                        }
                    }
                }
        }
        .navigationViewStyle(.stack)
    }
}

struct ShareExtScreenPreparingFiles: View {
    var vm: ShareExtScreenViewModel

    init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    public var body: some View {
        NavigationView {
            ProgressView("Preparing files")
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            vm.dismiss()
                        } label: {
                            Text("Cancel").bold()
                        }
                    }
                }
        }
        .navigationViewStyle(.stack)
    }
}

struct ShareExtScreenNoFiles: View {
    var vm: ShareExtScreenViewModel

    init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    public var body: some View {
        NavigationView {
            Text("No files to upload.")
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            vm.dismiss()
                        } label: {
                            Text("Dismiss").bold()
                        }
                    }
                }
        }
        .navigationViewStyle(.stack)
    }
}

struct ShareExtScreenDone: View {
    var vm: ShareExtScreenViewModel

    init(vm: ShareExtScreenViewModel) {
        self.vm = vm
    }

    @State private var progress: Double = 0

    public var body: some View {
        NavigationView {
            VStack {
                Text("Upload successful")
                    .font(.title)
                    .padding(.bottom, 30)

                ProgressView(value: progress, total: 100)
                    .tint(Color(.systemGray3))
            }
            .padding()
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        vm.dismiss()
                    } label: {
                        Text("Dismiss").bold()
                    }
                }
            }
        }
        .navigationViewStyle(.stack)
        .task {
            Task {
                progress = 0

                for _ in 0...99 {
                    progress += 1

                    try await Task.sleep(nanoseconds: 25_000_000)
                }

                vm.dismiss()
            }
        }
    }
}
