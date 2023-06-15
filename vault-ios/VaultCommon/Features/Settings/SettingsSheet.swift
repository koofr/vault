import SwiftUI
import VaultMobile

public struct SettingsSheet: View {
    public let container: Container
    public var onDismiss: () -> Void

    @ObservedObject private var user: Subscription<User>

    @State private var isLoggingOut: Bool = false
    @State private var isClearingCache: Bool = false

    public init(container: Container, onDismiss: @escaping () -> Void) {
        self.container = container
        self.onDismiss = onDismiss

        self.user = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.userSubscribe(cb: cb)
            },
            getData: { v, id in
                v.userData(id: id)
            })
    }

    public var body: some View {
        NavigationView {
            List {
                Section {
                    HStack {
                        UserIcon(container: container)
                        Spacer().frame(width: 15)
                        VStack(alignment: .leading, spacing: 3) {
                            Text(user.data?.fullName ?? "")
                            Text(user.data?.email ?? "").font(.system(size: 14)).foregroundColor(
                                Color(.systemGray))
                        }
                    }
                    .frame(height: 60)
                }

                Section {
                    Button {
                        container.sheets.show(name: "infoSheet") { _, hide in
                            InfoSheet(container: container, onDismiss: hide)
                        }
                    } label: {
                        VStack(alignment: .leading, spacing: 3) {
                            Text("Information").foregroundColor(Color(.label))
                            Text("Service and application information").font(.system(size: 12))
                                .foregroundColor(Color(.systemGray))
                        }
                    }
                }

                Section {
                    Button {
                        self.isClearingCache = true

                        Task.detached {
                            do {
                                try container.storageHelper.clearCache()

                                container.mobileVault.notificationsShow(
                                    message: "Cache has been cleared")
                            } catch {
                                container.mobileVault.notificationsShow(message: "\(error)")
                            }

                            Task { @MainActor in
                                self.isClearingCache = false
                            }
                        }
                    } label: {
                        Text("Clear cache").foregroundColor(Color(.label))
                    }
                    .disabled(isClearingCache)
                }

                Section {
                    Button {
                        container.authHelper.removeAccount()
                    } label: {
                        Text("Remove account…").foregroundColor(Color(.label))
                    }
                }

                Section {
                    Button {
                        isLoggingOut = true

                        container.authHelper.logout {
                            isLoggingOut = false
                        }
                    } label: {
                        Text("Logout").foregroundColor(Color(.label))
                    }
                    .disabled(isLoggingOut)
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitle("", displayMode: .inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        onDismiss()
                    } label: {
                        Text("Done").bold()
                    }
                }
            }
        }
        .onAppear {
            isLoggingOut = false
        }
    }
}
