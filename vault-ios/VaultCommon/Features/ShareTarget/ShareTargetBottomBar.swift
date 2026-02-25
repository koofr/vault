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
                Text(
                    LocalizedStringResource(
                        "ios.share_target.items_count.label",
                        defaultValue: "\(vm.files.count) items…",
                        bundle: #bundle,
                        comment:
                            "Bottom bar button text in share extension flow showing selected item count."
                    )
                )
            })
    }
}
