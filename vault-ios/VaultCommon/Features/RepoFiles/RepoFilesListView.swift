import SwiftUI
import VaultMobile

public let repoFilesListSummaryHeight = 80.0

public enum RepoFilesListItem {
    case item(item: RepoFilesBrowserItem)
    case summary

    var id: String {
        switch self {
        case .item(let item): return item.file.id
        case .summary: return "summary"
        }
    }
}

public struct RepoFilesListView: View {
    @ObservedObject var vm: RepoFilesScreenViewModel
    @ObservedObject var info: Subscription<RepoFilesBrowserInfo>

    private func getItems(_ browserItems: [RepoFilesBrowserItem]) -> [RepoFilesListItem] {
        var items = browserItems.map { RepoFilesListItem.item(item: $0) }

        if vm.editMode == .inactive {
            items.append(.summary)
        }

        return items
    }

    public var body: some View {
        if let info = info.data {
            GeometryReader { listGeometry in
                List(getItems(info.items), id: \.id, selection: $vm.selection) { item in
                    switch item {
                    case .item(let item):
                        RepoFilesListRow(vm: vm, item: item)
                            .frame(height: fileRowHeight)
                            .listRowInsets(
                                .init(
                                    top: fileRowInsetTop, leading: fileRowInsetLeading,
                                    bottom: fileRowInsetBottom,
                                    trailing: fileRowInsetTrailing))
                    case .summary:
                        RepoFilesListSummary(
                            vm: vm, itemsCount: info.items.count,
                            listHeight: listGeometry.size.height)
                    }
                }
            }
        }
    }
}

struct RepoFilesListSummary: View {
    let vm: RepoFilesScreenViewModel
    let itemsCount: Int
    let listHeight: Double

    init(vm: RepoFilesScreenViewModel, itemsCount: Int, listHeight: Double) {
        self.vm = vm
        self.itemsCount = itemsCount
        self.listHeight = listHeight
    }

    func getSummaryHeight(itemsCount: Int, listHeight: Double) -> Double {
        let itemsHeight =
            Double(itemsCount) * (fileRowHeight + fileRowInsetTop + fileRowInsetBottom)

        if itemsHeight + repoFilesListSummaryHeight > listHeight {
            return repoFilesListSummaryHeight
        } else {
            return listHeight - itemsHeight
        }
    }

    var body: some View {
        // summary needs to be a button otherwise it can be selected
        Button {

        } label: {
            VStack {
                Spacer()
                HStack {
                    Spacer()
                    Text(itemsCount == 1 ? "1 item" : "\(itemsCount) items").bold().padding()
                    Spacer()
                }
                .frame(height: repoFilesListSummaryHeight)
            }
        }
        .frame(height: getSummaryHeight(itemsCount: itemsCount, listHeight: listHeight))
        .listRowSeparator(.hidden)
        .listRowInsets(.init(top: 0, leading: 0, bottom: 0, trailing: 0))
        .contextMenu {
            RepoFilesListSummaryMenu(vm: vm)
        }
    }
}

struct RepoFilesListSummaryMenu: View {
    let vm: RepoFilesScreenViewModel

    init(vm: RepoFilesScreenViewModel) {
        self.vm = vm
    }

    var body: some View {
        Button {
            vm.container.mobileVault.repoFilesBrowsersCreateDir(
                browserId: vm.browserId, cb: RepoFilesBrowserDirCreatedFn { _ in })
        } label: {
            Label("New folder", systemImage: "folder.badge.plus")
        }

        PasteButton(supportedContentTypes: uploadHelperUTTypes) { itemProviders in
            Task {
                let files = await vm.container.uploadHelper.itemProvidersToFiles(
                    itemProviders: itemProviders, loadFileRepresentation: false)

                vm.container.uploadHelper.uploadFiles(
                    repoId: vm.repoId, encryptedParentPath: vm.encryptedPath, files: files)
            }
        }
    }
}
