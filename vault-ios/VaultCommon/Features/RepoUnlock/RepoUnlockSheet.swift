import SwiftUI

public struct RepoUnlockSheet: View {
    @ObservedObject public var vm: RepoUnlockScreenViewModel
    public let onDismiss: () -> Void

    init(vm: RepoUnlockScreenViewModel, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.onDismiss = onDismiss
    }

    public var body: some View {
        NavigationView {
            RepoUnlockScreen(vm: vm, onUnlock: onDismiss)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button {
                            onDismiss()
                        } label: {
                            Text("Cancel")
                        }
                    }
                }
        }
    }
}
