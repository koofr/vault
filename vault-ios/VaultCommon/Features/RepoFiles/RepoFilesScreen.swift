import Combine
import SwiftUI
import UniformTypeIdentifiers
import VaultMobile

public struct RepoFilesScreen: View {
    @ObservedObject var vm: RepoFilesScreenViewModel

    @ObservedObject private var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    private var navigationTitle: String {
        switch vm.editMode {
        case .active:
            if let selectedCount = info.data?.selectedCount {
                return selectedCount == 0
                    ? "Selected items" : selectedCount == 1 ? "1 item" : "\(selectedCount) items"
            } else {
                return "Selected items"
            }
        default:
            return info.data?.title ?? ""
        }
    }

    public init(vm: RepoFilesScreenViewModel) {
        self.vm = vm

        self.info = vm.info
    }

    public var body: some View {
        Group {
            if let info = info.data {
                RefreshableList(
                    status: info.status,
                    isEmpty: info.items.isEmpty,
                    onRefresh: {
                        vm.container.mobileVault.repoFilesBrowsersLoadFiles(
                            browserId: vm.browserId)
                    },
                    empty: {
                        EmptyFolderView()
                            .contextMenu {
                                RepoFilesListSummaryMenu(vm: vm)
                            }
                    }
                ) {
                    RepoFilesListView(vm: vm, info: self.info)
                }
                .listStyle(.plain)
            }
        }
        .environment(\.editMode, $vm.editMode)
        .animation(.default, value: vm.editMode)
        .navigationTitle(navigationTitle)
        .navigationBarTitleDisplayMode(.inline)
        .navigationBarBackButtonHidden(vm.editMode == .active)
        .toolbar {
            if vm.editMode == .active {
                ToolbarItem(placement: .navigationBarLeading) {
                    RepoFilesSelectionButton(vm: vm)
                }
            }

            ToolbarItem(placement: .navigationBarTrailing) {
                TransfersButton(container: vm.container)
            }

            if vm.editMode == .active {
                ToolbarItem(placement: .confirmationAction) {
                    RepoFilesStopEditButton(vm: vm)
                }
            }

            if vm.editMode == .inactive {
                ToolbarItem(placement: .primaryAction) {
                    RepoFilesNavMenuButton(vm: vm)
                }
            }

            if vm.editMode == .active {
                ToolbarItem(placement: .bottomBar) {
                    RepoFilesEditModeBottomBar(vm: vm)
                }
            }
        }
    }
}

struct RepoFilesSelectionButton: View {
    let vm: RepoFilesScreenViewModel

    @ObservedObject var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm

        self.info = vm.info
    }

    var body: some View {
        Button {
            if info.data?.selectionSummary == .all {
                vm.container.mobileVault.repoFilesBrowsersClearSelection(browserId: vm.browserId)
            } else {
                vm.container.mobileVault.repoFilesBrowsersSelectAll(browserId: vm.browserId)
            }
        } label: {
            Text(info.data?.selectionSummary == .all ? "Deselect All" : "Select All")
        }
    }
}

struct RepoFilesStopEditButton: View {
    var vm: RepoFilesScreenViewModel

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm
    }

    var body: some View {
        Button {
            vm.stopEditMode()
        } label: {
            Text("Done")
        }
    }
}

struct RepoFilesNavMenuButton: View {
    @ObservedObject var vm: RepoFilesScreenViewModel

    @ObservedObject var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm

        self.info = vm.info
    }

    var body: some View {
        Menu {
            RepoFilesNavMenu(vm: vm)
        } label: {
            Image(systemName: "ellipsis.circle")
        }
        .fileImporter(
            isPresented: $vm.filesImporterIsPresented,
            allowedContentTypes: vm.filesImporterAllowedContentTypes,
            allowsMultipleSelection: vm.filesImporterAllowsMultipleSelection
        ) { result in
            switch result {
            case .success(let urls):
                Task(priority: .background) {
                    do {
                        try vm.container.uploadHelper.uploadSecurityScopedResources(
                            repoId: vm.repoId, parentPath: vm.path, urls: urls)
                    } catch {
                        vm.container.mobileVault.notificationsShow(message: "\(error)")
                    }
                }
            case .failure(let error):
                vm.container.mobileVault.notificationsShow(message: "\(error)")
            }
        }
    }
}

struct RepoFilesNavMenu: View {
    let vm: RepoFilesScreenViewModel

    @ObservedObject var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm

        self.info = vm.info
    }

    var body: some View {
        if let info = info.data {
            if info.totalCount > 0 {
                Button {
                    vm.startEditMode()
                } label: {
                    Label("Select", systemImage: "checkmark.circle")
                }
            }
        }

        Button {
            vm.container.mobileVault.repoFilesBrowsersCreateDir(
                browserId: vm.browserId, cb: RepoFilesBrowserDirCreatedFn { _ in })
        } label: {
            Label("New folder", systemImage: "folder.badge.plus")
        }

        Button {
            vm.container.sheets.show(name: "repoFilesImagePicker") { _, hide in
                RepoFilesImagePicker(vm: vm, onDismiss: hide)
            }
        } label: {
            Label("Upload photo", systemImage: "photo")
        }

        Button {
            vm.uploadFiles()
        } label: {
            Label("Upload files", systemImage: "doc.on.doc")
        }

        Button {
            vm.uploadFolder()
        } label: {
            Label("Upload a folder", systemImage: "folder")
        }

        Divider()

        if let info = info.data {
            let (items, selected) = RepoFilesSortFieldItem.getItems(selected: info.sort.field)

            let pickerSelection = Binding(
                get: { selected },
                set: { item in
                    vm.container.mobileVault.repoFilesBrowsersSortBy(
                        browserId: vm.browserId, field: item.field, direction: nil)
                })
            let sortImage = info.sort.direction == .asc ? "chevron.up" : "chevron.down"

            Picker("Sort by", selection: pickerSelection) {
                ForEach(items, id: \.self) { item in
                    if item == selected {
                        Label(item.text, systemImage: sortImage)
                    } else {
                        Text(item.text)
                    }
                }
            }
        }
    }
}

struct RepoFilesSortFieldItem: Equatable, Hashable {
    let field: RepoFilesSortField
    let text: String

    static func getItems(selected: RepoFilesSortField) -> (
        [RepoFilesSortFieldItem], RepoFilesSortFieldItem
    ) {
        let nameItem = RepoFilesSortFieldItem(field: RepoFilesSortField.name, text: "Name")
        let sizeItem = RepoFilesSortFieldItem(field: RepoFilesSortField.size, text: "Size")
        let modifiedItem = RepoFilesSortFieldItem(
            field: RepoFilesSortField.modified, text: "Modified")

        let items = [nameItem, sizeItem, modifiedItem]

        switch selected {
        case .name: return (items, nameItem)
        case .size: return (items, sizeItem)
        case .modified: return (items, modifiedItem)
        }
    }
}

struct RepoFilesEditModeBottomBar: View {
    let vm: RepoFilesScreenViewModel

    @ObservedObject var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm

        self.info = vm.info
    }

    var body: some View {
        let hasSelection = (info.data?.selectedCount ?? 0) > 0

        HStack {
            Button {
                vm.container.downloadHelper.downloadRepoFilesBrowsersSelected(
                    browserId: vm.browserId)
            } label: {
                Image(systemName: "arrow.down.to.line.compact")
            }
            .disabled(!hasSelection)
            .accessibilityLabel("Download selected")

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersMoveSelected(
                    browserId: vm.browserId, mode: RepoFilesMoveMode.copy)
            } label: {
                Image(systemName: "doc.on.doc")
            }
            .disabled(!hasSelection)
            .accessibilityLabel("Copy selected")

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersMoveSelected(
                    browserId: vm.browserId, mode: RepoFilesMoveMode.move)
            } label: {
                Image(systemName: "folder")
            }
            .disabled(!hasSelection)
            .accessibilityLabel("Move selected")

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersDeleteSelected(browserId: vm.browserId)
            } label: {
                Image(systemName: "trash")
            }
            .disabled(!hasSelection)
            .accessibilityLabel("Delete selected")
        }
    }
}
