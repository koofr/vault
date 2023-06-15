import SwiftUI
import VaultMobile

public class RemoteFilesDirPickerScreenViewModel: ObservableObject {
    public let container: Container
    @Published public var dirPickerVm: RemoteFilesDirPickerViewModel
    public let location: String

    public let browserId: UInt32

    @Published public var navController: RemoteFilesDirPickerNavController

    public init(container: Container, dirPickerVm: RemoteFilesDirPickerViewModel, location: String)
    {
        self.container = container
        self.dirPickerVm = dirPickerVm
        self.location = location

        browserId = container.mobileVault.remoteFilesBrowsersCreate(
            location: location,
            options: RemoteFilesBrowserOptions(selectName: nil, onlyHostedDevices: true))

        navController = dirPickerVm.navController
    }

    deinit {
        container.mobileVault.remoteFilesBrowsersDestroy(browserId: browserId)
    }
}

public struct RemoteFilesDirPickerScreen: View {
    @ObservedObject var vm: RemoteFilesDirPickerScreenViewModel

    @ObservedObject var navController: RemoteFilesDirPickerNavController

    @ObservedObject private var info: Subscription<RemoteFilesBrowserInfo>

    public var canSelect: Bool {
        if navController.state.isNavigating {
            return false
        }

        if let info = info.data {
            if let mountId = info.mountId {
                if let path = info.path {
                    return vm.dirPickerVm.canSelect(mountId, path)
                }
            }
        }

        return false
    }

    public init(vm: RemoteFilesDirPickerScreenViewModel) {
        self.vm = vm

        self.navController = vm.navController

        info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.remoteFilesBrowsersInfoSubscribe(browserId: vm.browserId, cb: cb)
            },
            getData: { v, id in
                v.remoteFilesBrowsersInfoData(id: id)
            })
    }

    public var body: some View {
        Group {
            if let info = info.data {
                RefreshableList(
                    status: info.status, isEmpty: info.items.isEmpty,
                    onRefresh: {
                        vm.container.mobileVault.remoteFilesBrowsersLoad(browserId: vm.browserId)
                    },
                    empty: {
                        EmptyFolderView()
                    }
                ) {
                    List(info.items, id: \.id) { item in
                        RemoteFilesDirPickerListRow(vm: vm, item: item)
                    }
                }
                .listStyle(.plain)
            }
        }
        .navigationTitle(info.data?.title ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if vm.location == "" {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        vm.dirPickerVm.onCancel()
                    } label: {
                        Text("Cancel")
                    }
                }
            }

            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button(
                        action: {
                            vm.container.mobileVault.remoteFilesBrowsersCreateDir(
                                browserId: vm.browserId,
                                cb: RemoteFilesBrowserDirCreatedFn { location in
                                    navController.push(.dirPicker(location: location))
                                })
                        },
                        label: {
                            Label("New folder", systemImage: "folder.badge.plus")
                        }
                    )
                    .disabled(info.data?.canCreateDir != true)
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }

            ToolbarItem(placement: .confirmationAction) {
                Button(
                    action: {
                        if let info = info.data {
                            if let mountId = info.mountId {
                                if let path = info.path {
                                    vm.dirPickerVm.onSelect(mountId, path)
                                }
                            }
                        }
                    },
                    label: {
                        Text("Select")
                    }
                )
                .disabled(!canSelect)
            }
        }
    }
}
