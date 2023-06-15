import SwiftUI
import VaultMobile

public struct ShareTargetBottomBar: View {
    @ObservedObject var vm: ShareTargetViewModel

    public var body: some View {
        Button(
            action: {
                vm.container.sheets.show(
                    viewModel: vm,
                    content: { vm, hide in
                        ShareTargetFilesSheet(vm: vm, onClose: hide)
                    })
            },
            label: {
                Text(vm.files.count == 1 ? "\(vm.files.count) item…" : "\(vm.files.count) items…")
            })
    }
}
