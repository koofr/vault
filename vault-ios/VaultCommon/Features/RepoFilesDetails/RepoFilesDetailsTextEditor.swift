import SwiftUI
import VaultMobile

struct RepoFilesDetailsTextEditor: View {
    @ObservedObject var vm: RepoFilesDetailsScreenViewModel

    @ObservedObject private var info: Subscription<RepoFilesDetailsInfo>

    @ObservedObject private var contentString: Subscription<Data>

    @FocusState private var isEditorFocused: Bool

    public init(vm: RepoFilesDetailsScreenViewModel) {
        self.vm = vm

        info = vm.info

        contentString = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsContentBytesSubscribe(detailsId: vm.detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsContentBytesData(id: id)
            })

        contentString.setOnData { data in
            if let data = data {
                if let dataString = String(data: data, encoding: .utf8) {
                    if dataString != vm.textEditorText {
                        DispatchQueue.main.async {
                            // Fix for "Publishing changes from within view updates is not allowed, this will cause undefined behavior."
                            vm.textEditorText = dataString
                        }
                    }
                }
            }
        }
    }

    var body: some View {
        if let info = info.data {
            switch info.contentStatus {
            case .err(let error, false):
                ErrorView(
                    errorText: error,
                    onRetry: {
                        vm.container.mobileVault.repoFilesDetailsLoadContent(
                            detailsId: vm.detailsId)
                    }
                ).frame(maxHeight: .infinity)
            case .initial, .loading(false):
                LoadingView().frame(maxHeight: .infinity)
            default:
                if info.isEditing {
                    let textBinding = Binding(
                        get: { vm.textEditorText },
                        set: { value in
                            vm.textEditorText = value

                            if let data = value.data(using: .utf8) {
                                vm.container.mobileVault.repoFilesDetailsSetContent(
                                    detailsId: vm.detailsId, content: data)
                            }
                        })

                    TextEditor(text: textBinding)
                        .font(Font(ReadOnlyTextEditor.monospacedFont()))
                        .focused($isEditorFocused)
                        .onAppear {
                            isEditorFocused = true
                        }
                        .padding()
                } else {
                    ReadOnlyTextEditor(text: vm.textEditorText)
                        .padding()
                }
            }
        }
    }
}
