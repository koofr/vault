import PhotosUI
import SwiftUI
import VaultMobile

public struct RepoFilesImagePicker: UIViewControllerRepresentable {
    public let vm: RepoFilesScreenViewModel
    public let onDismiss: () -> Void

    public init(vm: RepoFilesScreenViewModel, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.onDismiss = onDismiss
    }

    public func makeUIViewController(context: Context) -> PHPickerViewController {
        var config = PHPickerConfiguration()
        config.preferredAssetRepresentationMode = .current
        config.selectionLimit = 0

        let picker = PHPickerViewController(configuration: config)
        picker.delegate = context.coordinator

        return picker
    }

    public func updateUIViewController(_ picker: PHPickerViewController, context: Context) {
        picker.delegate = context.coordinator
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(vm: vm, onDismiss: onDismiss)
    }

    public class Coordinator: NSObject, PHPickerViewControllerDelegate {
        public let vm: RepoFilesScreenViewModel
        public let onDismiss: () -> Void

        public init(vm: RepoFilesScreenViewModel, onDismiss: @escaping () -> Void) {
            self.vm = vm
            self.onDismiss = onDismiss
        }

        public func picker(
            _ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]
        ) {
            onDismiss()

            if !results.isEmpty {
                Task {
                    await uploadResults(results)
                }
            }
        }

        @MainActor
        private func uploadResults(_ results: [PHPickerResult]) async {
            let itemProviders = results.map { $0.itemProvider }

            let files = await vm.container.uploadHelper.itemProvidersToFiles(
                itemProviders: itemProviders, loadFileRepresentation: true)

            self.vm.container.uploadHelper.uploadFiles(
                repoId: vm.repoId, parentPath: vm.path, files: files)
        }
    }
}
