import Combine
import SwiftUI
import UniformTypeIdentifiers
import VaultMobile

public struct RepoFilesScreen: View {
    @ObservedObject var vm: RepoFilesScreenViewModel

    @ObservedObject private var info: VaultMobile.Subscription<RepoFilesBrowserInfo>

    private var navigationTitle: Text {
        switch vm.editMode {
        case .active:
            if let selectedCount = info.data?.selectedCount {
                return selectedCount == 0
                    ? Text(
                        LocalizedStringResource(
                            "ios.repo_files.edit_mode.title",
                            defaultValue: "Selected items",
                            bundle: #bundle,
                            comment:
                                "Navigation title in file browser edit mode when no items are selected."
                        )
                    )
                    : Text(
                        LocalizedStringResource(
                            "ios.repo_files.edit_mode.items_count.label",
                            defaultValue: "\(selectedCount) items",
                            bundle: #bundle,
                            comment:
                                "Navigation title in file browser edit mode showing selected item count."
                        )
                    )
            } else {
                return Text(
                    LocalizedStringResource(
                        "ios.repo_files.edit_mode.title",
                        defaultValue: "Selected items",
                        bundle: #bundle,
                        comment:
                            "Navigation title in file browser edit mode when no items are selected."
                    )
                )
            }
        default:
            return Text(info.data?.title ?? "")
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
                            .if(info.encryptedPath != nil) { view in
                                view.contextMenu {
                                    RepoFilesListSummaryMenu(vm: vm)
                                }
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
            Text(
                info.data?.selectionSummary == .all
                    ? LocalizedStringResource(
                        "ios.repo_files.edit_mode.deselect_all.button",
                        defaultValue: "Deselect All",
                        bundle: #bundle,
                        comment: "Toolbar button in file browser edit mode to clear selection."
                    )
                    : LocalizedStringResource(
                        "ios.repo_files.edit_mode.select_all.button",
                        defaultValue: "Select All",
                        bundle: #bundle,
                        comment:
                            "Toolbar button in file browser edit mode to select all visible items."
                    )
            )
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
            Text(
                LocalizedStringResource(
                    "ios.repo_files.edit_mode.done.button",
                    defaultValue: "Done",
                    bundle: #bundle,
                    comment: "Toolbar button in file browser edit mode to exit selection mode."
                )
            )
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
                        if let info = vm.info.data {
                            if let repoId = info.repoId {
                                if let encryptedPath = info.encryptedPath {
                                    try vm.container.uploadHelper.uploadSecurityScopedResources(
                                        repoId: repoId, encryptedParentPath: encryptedPath,
                                        urls: urls)
                                }
                            }
                        }
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

    @Environment(\.locale) private var locale

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
                    Label {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_files.nav_menu.select.menu_item",
                                defaultValue: "Select",
                                bundle: #bundle,
                                comment:
                                    "Navigation menu item in file browser that enters multi-select mode."
                            )
                        )
                    } icon: {
                        Image(systemName: "checkmark.circle")
                    }
                }
            }

            if let repoId = info.repoId {
                if info.encryptedPath != nil {
                    Button {
                        vm.container.mobileVault.repoFilesBrowsersCreateDir(
                            browserId: vm.browserId, cb: RepoFilesBrowserDirCreatedFn { _ in })
                    } label: {
                        Label {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files.nav_menu.new_folder.menu_item",
                                    defaultValue: "New folder",
                                    bundle: #bundle,
                                    comment:
                                        "Navigation menu item in file browser that creates a new folder."
                                )
                            )
                        } icon: {
                            Image(systemName: "folder.badge.plus")
                        }
                    }

                    Button {
                        vm.container.sheets.show(name: "repoFilesImagePicker") { _, hide in
                            RepoFilesImagePicker(vm: vm, onDismiss: hide)
                        }
                    } label: {
                        Label {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files.nav_menu.upload_photo.menu_item",
                                    defaultValue: "Upload photo",
                                    bundle: #bundle,
                                    comment:
                                        "Navigation menu item in file browser that opens photo picker upload."
                                )
                            )
                        } icon: {
                            Image(systemName: "photo")
                        }
                    }

                    Button {
                        vm.uploadFiles()
                    } label: {
                        Label {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files.nav_menu.upload_files.menu_item",
                                    defaultValue: "Upload files",
                                    bundle: #bundle,
                                    comment:
                                        "Navigation menu item in file browser that opens file picker upload."
                                )
                            )
                        } icon: {
                            Image(systemName: "doc.on.doc")
                        }
                    }

                    Button {
                        vm.uploadFolder()
                    } label: {
                        Label {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files.nav_menu.upload_folder.menu_item",
                                    defaultValue: "Upload a folder",
                                    bundle: #bundle,
                                    comment:
                                        "Navigation menu item in file browser that uploads a folder."
                                )
                            )
                        } icon: {
                            Image(systemName: "folder")
                        }
                    }

                    Button {
                        let formatter = DateFormatter()
                        formatter.dateFormat = "yyyyMMddHHmmss"
                        let date = formatter.string(from: Date())

                        let name =
                            String(
                                localized: LocalizedStringResource(
                                    "ios.repo_files.create_text_file.default_filename",
                                    defaultValue: "new text file \(date)",
                                    locale: locale,
                                    bundle: #bundle,
                                    comment:
                                        "Default base filename used when creating a new text file in a folder."
                                )) + ".txt"

                        vm.container.mobileVault.repoFilesBrowsersCreateFile(
                            browserId: vm.browserId, name: name,
                            cb: RepoFilesBrowserFileCreatedFn { encryptedPath in
                                vm.navController.push(
                                    .repoFilesDetails(
                                        repoId: repoId, encryptedPath: encryptedPath,
                                        isEditing: true))
                            })
                    } label: {
                        Label {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files.nav_menu.create_text_file.menu_item",
                                    defaultValue: "Create new text file",
                                    bundle: #bundle,
                                    comment:
                                        "Navigation menu item in file browser that creates a new text file."
                                )
                            )
                        } icon: {
                            Image(systemName: "doc.badge.plus")
                        }
                    }
                }
            }

            Divider()

            let (items, selected) = RepoFilesSortFieldItem.getItems(selected: info.sort.field)

            let pickerSelection = Binding(
                get: { selected },
                set: { item in
                    vm.container.mobileVault.repoFilesBrowsersSortBy(
                        browserId: vm.browserId, field: item.field, direction: nil)
                })
            let sortImage = info.sort.direction == .asc ? "chevron.up" : "chevron.down"

            Picker(
                selection: pickerSelection,
                label: Text(
                    LocalizedStringResource(
                        "ios.repo_files.nav_menu.sort_by.menu_item",
                        defaultValue: "Sort by",
                        bundle: #bundle,
                        comment: "Label for sort field picker in the file browser navigation menu."
                    )
                )
            ) {
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
    let text: LocalizedStringResource

    static func == (lhs: RepoFilesSortFieldItem, rhs: RepoFilesSortFieldItem) -> Bool {
        lhs.field == rhs.field
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(field)
    }

    static func getItems(selected: RepoFilesSortField) -> (
        [RepoFilesSortFieldItem], RepoFilesSortFieldItem
    ) {
        let nameItem = RepoFilesSortFieldItem(
            field: RepoFilesSortField.name,
            text: LocalizedStringResource(
                "ios.repo_files.sort_field.name",
                defaultValue: "Name",
                bundle: #bundle,
                comment: "Sort option label in file browser for sorting by file name."
            )
        )
        let sizeItem = RepoFilesSortFieldItem(
            field: RepoFilesSortField.size,
            text: LocalizedStringResource(
                "ios.repo_files.sort_field.size",
                defaultValue: "Size",
                bundle: #bundle,
                comment: "Sort option label in file browser for sorting by file size."
            )
        )
        let modifiedItem = RepoFilesSortFieldItem(
            field: RepoFilesSortField.modified,
            text: LocalizedStringResource(
                "ios.repo_files.sort_field.modified",
                defaultValue: "Modified",
                bundle: #bundle,
                comment: "Sort option label in file browser for sorting by modified date."
            )
        )

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
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.repo_files.edit_mode.download_selected.a11y.label",
                        defaultValue: "Download selected",
                        bundle: #bundle,
                        comment:
                            "Accessibility label for edit mode bottom-bar button that downloads selected items."
                    )
                )
            )
            .padding()

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersMoveSelected(
                    browserId: vm.browserId, mode: RepoFilesMoveMode.copy)
            } label: {
                Image(systemName: "doc.on.doc")
            }
            .disabled(!hasSelection)
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.repo_files.edit_mode.copy_selected.a11y.label",
                        defaultValue: "Copy selected",
                        bundle: #bundle,
                        comment:
                            "Accessibility label for edit mode bottom-bar button that copies selected items."
                    )
                )
            )
            .padding()

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersMoveSelected(
                    browserId: vm.browserId, mode: RepoFilesMoveMode.move)
            } label: {
                Image(systemName: "folder")
            }
            .disabled(!hasSelection)
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.repo_files.edit_mode.move_selected.a11y.label",
                        defaultValue: "Move selected",
                        bundle: #bundle,
                        comment:
                            "Accessibility label for edit mode bottom-bar button that moves selected items."
                    )
                )
            )
            .padding()

            Spacer()

            Button {
                vm.container.mobileVault.repoFilesBrowsersDeleteSelected(browserId: vm.browserId)
            } label: {
                Image(systemName: "trash")
            }
            .disabled(!hasSelection)
            .accessibilityLabel(
                Text(
                    LocalizedStringResource(
                        "ios.repo_files.edit_mode.delete_selected.a11y.label",
                        defaultValue: "Delete selected",
                        bundle: #bundle,
                        comment:
                            "Accessibility label for edit mode bottom-bar button that deletes selected items."
                    )
                )
            )
            .padding()
        }
    }
}
