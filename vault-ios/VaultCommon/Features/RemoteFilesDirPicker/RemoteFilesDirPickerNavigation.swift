import SwiftUI
import SwiftUINavController
import VaultMobile

public struct RemoteFilesDirPickerNavigation: View {
    public let vm: RemoteFilesDirPickerViewModel

    public var body: some View {
        Navigation(navController: vm.navController) { navController, routeContainer in
            switch routeContainer.route {
            case .dirPicker(let location):
                RemoteFilesDirPickerScreen(
                    vm: navController.ensureViewModel(routeContainer: routeContainer) {
                        RemoteFilesDirPickerScreenViewModel(
                            container: vm.container, dirPickerVm: vm, location: location)
                    })
            }
        }
    }
}
