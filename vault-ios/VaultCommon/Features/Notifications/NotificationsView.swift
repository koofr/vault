import SwiftUI
import VaultMobile

public struct NotificationsView: View {
    private let container: Container

    @ObservedObject private var notifications: Subscription<[VaultMobile.Notification]>

    public init(container: Container) {
        self.container = container

        self.notifications = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.notificationsSubscribe(cb: cb)
            },
            getData: { v, id in
                v.notificationsData(id: id)
            })
    }

    public var body: some View {
        VStack(spacing: 0) {
            Spacer()

            VStack(spacing: 0) {
                ForEach(notifications.data!, id: \.id) {
                    let notification = $0

                    NotificationView(message: notification.message)
                        .onTapGesture {
                            container.mobileVault.notificationsRemove(
                                notificationId: notification.id)
                        }
                        .onAppear {
                            container.mobileVault.notificationsRemoveAfter(
                                notificationId: notification.id, durationMs: 3000)
                        }
                }
            }
            .offset(y: -20)
        }
    }
}
