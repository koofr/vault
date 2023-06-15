import SwiftUI
import VaultMobile

public struct RemoteFilesDirPickerListRow: View {
    @ObservedObject var vm: RemoteFilesDirPickerScreenViewModel
    public var item: RemoteFilesBrowserItem

    public var canNavigate: Bool {
        switch item.typ {
        case .file(typ: .dir, _): return true
        case .file(typ: .file, _): return false
        default: return true
        }
    }

    public var body: some View {
        if canNavigate {
            Button {
                vm.navController.push(.dirPicker(location: item.id))
            } label: {
                RemoteFilesBrowserItemRow(container: vm.container, item: item)
            }
        } else {
            RemoteFilesBrowserItemRow(container: vm.container, item: item)
        }
    }
}
