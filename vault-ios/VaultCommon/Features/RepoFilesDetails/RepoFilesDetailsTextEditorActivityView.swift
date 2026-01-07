import SwiftUI
import VaultMobile

struct RepoFilesDetailsTextEditorActivityView: View {
    @ObservedObject var vm: RepoFilesDetailsScreenViewModel

    @ObservedObject private var contentString: Subscription<Data>

    public init(vm: RepoFilesDetailsScreenViewModel) {
        self.vm = vm

        contentString = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesDetailsContentBytesSubscribe(detailsId: vm.detailsId, cb: cb)
            },
            getData: { v, id in
                v.repoFilesDetailsContentBytesData(id: id)
            })
    }

    public var body: some View {
        if let data = contentString.data {
            if let dataString = String(data: data, encoding: .utf8) {
                ActivityView(activityItems: [dataString], showOpenInDownloads: false)
            }
        }
    }
}
