import SwiftUI
import VaultMobile

public struct SettingsSheet: View {
    public let container: Container
    public var onDismiss: () -> Void

    @ObservedObject private var user: Subscription<User>

    @State private var isLoggingOut: Bool = false
    @State private var isClearingCache: Bool = false

    @Environment(\.locale) private var locale

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
                            Text(
                                LocalizedStringResource(
                                    "ios.settings.information.label",
                                    defaultValue: "Information",
                                    bundle: #bundle,
                                    comment:
                                        "Primary label for the row that opens the information sheet in settings."
                                )
                            )
                            .foregroundColor(Color(.label))
                            Text(
                                LocalizedStringResource(
                                    "ios.settings.information.description",
                                    defaultValue: "Service and application information",
                                    bundle: #bundle,
                                    comment:
                                        "Secondary description under the Information row in settings."
                                )
                            )
                            .font(.system(size: 12))
                            .foregroundColor(Color(.systemGray))
                        }
                    }
                }

                Section {
                    Button {
                        container.sheets.show(
                            name: "languagePicker",
                            viewModel: LanguagePickerViewModel(container: container)
                        ) { vm, hide in
                            LanguagePickerSheet(
                                vm: vm,
                                onDismiss: hide)
                        }
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.change_language.label",
                                defaultValue: "Change language",
                                bundle: #bundle,
                                comment: "Row label in settings that opens the language picker."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }
                }

                Section {
                    Button {
                        self.isClearingCache = true

                        Task.detached {
                            do {
                                try await container.storageHelper.clearCache()

                                await container.mobileVault.notificationsShow(
                                    message: String(
                                        localized: LocalizedStringResource(
                                            "ios.settings.clear_cache.success",
                                            defaultValue: "Cache has been cleared",
                                            locale: locale,
                                            bundle: #bundle,
                                            comment:
                                                "Toast message shown after cache clearing succeeds from settings."
                                        )
                                    ))
                            } catch {
                                await container.mobileVault.notificationsShow(message: "\(error)")
                            }

                            Task { @MainActor in
                                self.isClearingCache = false
                            }
                        }
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.clear_cache.label",
                                defaultValue: "Clear cache",
                                bundle: #bundle,
                                comment: "Row label in settings that clears locally cached data."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }
                    .disabled(isClearingCache)
                }

                Section {
                    Button {
                        container.authHelper.removeAccount()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.remove_account.label",
                                defaultValue: "Remove account…",
                                bundle: #bundle,
                                comment:
                                    "Row label in settings that starts the account removal flow."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }
                }

                Section {
                    Button {
                        isLoggingOut = true

                        container.authHelper.logout {
                            isLoggingOut = false
                        }
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.logout.label",
                                defaultValue: "Logout",
                                bundle: #bundle,
                                comment: "Row label in settings that logs the user out."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }
                    .disabled(isLoggingOut)
                }
            }
            .navigationTitle(
                Text(
                    LocalizedStringResource(
                        "ios.settings.title",
                        defaultValue: "Settings",
                        bundle: #bundle,
                        comment: "Navigation title of the settings sheet."
                    )
                )
            )
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        onDismiss()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.done.button",
                                defaultValue: "Done",
                                bundle: #bundle,
                                comment: "Toolbar button that dismisses the settings sheet."
                            )
                        )
                        .bold()
                    }
                }
            }
        }
        .onAppear {
            isLoggingOut = false
        }
    }
}
