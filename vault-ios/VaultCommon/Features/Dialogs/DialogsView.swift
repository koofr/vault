import SwiftUI
import VaultMobile

public struct DialogsView: View {
    public let container: Container

    @ObservedObject private var dialogs: Subscription<[UInt32]>

    public init(container: Container) {
        self.container = container

        dialogs = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.dialogsSubscribe(cb: cb)
            },
            getData: { v, id in
                v.dialogsData(id: id)
            })
    }

    public var body: some View {
        Group {
            if let dialogs = dialogs.data {
                ForEach(dialogs, id: \.self) {
                    DialogView(container: container, dialogId: $0)
                }
            }
        }
    }
}
