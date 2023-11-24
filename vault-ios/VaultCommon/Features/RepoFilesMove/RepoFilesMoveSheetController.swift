import Foundation
import VaultMobile

public class RepoFilesMoveSheetController {
    private weak var container: Container?

    private var repoFilesMoveInfo: Subscription<RepoFilesMoveInfo>?

    public init() {}

    public func setContainer(container: Container) {
        self.container = container

        let repoFilesMoveInfo = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.repoFilesMoveInfoSubscribe(cb: cb)
            },
            getData: { v, id in
                v.repoFilesMoveInfoData(id: id)
            })

        repoFilesMoveInfo.setOnData { info in
            if container.sheets.isVisible(name: "repoFilesMove") {
                if info == nil {
                    container.sheets.hide(name: "repoFilesMove")
                }
            } else {
                if let info = info {
                    container.sheets.show(
                        name: "repoFilesMove",
                        viewModel: RepoFilesMoveViewModel(
                            container: container, repoId: info.repoId,
                            initialEncryptedPathChain: info.encryptedDestPathChain),
                        onHide: {
                            container.mobileVault.repoFilesMoveCancel()
                        }
                    ) { vm, _ in
                        RepoFilesMoveNavigation(vm: vm)
                    }
                }
            }
        }

        self.repoFilesMoveInfo = repoFilesMoveInfo
    }
}
