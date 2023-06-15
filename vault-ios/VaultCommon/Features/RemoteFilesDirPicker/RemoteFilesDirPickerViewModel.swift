import Foundation
import VaultMobile

public class RemoteFilesDirPickerViewModel: ObservableObject {
    public let container: Container
    public let canSelect: (String, String) -> Bool
    public let onSelect: (String, String) -> Void
    public let onCancel: () -> Void

    public let navController: RemoteFilesDirPickerNavController

    public init(
        container: Container, canSelect: @escaping (String, String) -> Bool,
        onSelect: @escaping (String, String) -> Void, onCancel: @escaping () -> Void
    ) {
        self.container = container
        self.canSelect = canSelect
        self.onSelect = onSelect
        self.onCancel = onCancel

        self.navController = NavController(rootRoute: .dirPicker(location: ""))
    }
}
