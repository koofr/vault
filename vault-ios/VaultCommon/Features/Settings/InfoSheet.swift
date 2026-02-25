import SwiftUI
import VaultMobile

public struct InfoSheet: View {
    public let container: Container
    public var onDismiss: () -> Void

    private var version: Version

    public init(container: Container, onDismiss: @escaping () -> Void) {
        self.container = container
        self.onDismiss = onDismiss

        self.version = container.mobileVault.version()
    }

    public var body: some View {
        NavigationView {
            List {
                Section {
                    Link(destination: URL(string: container.baseURL)!) {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(String(stringLiteral: "Koofr Vault"))
                                .foregroundColor(Color(.label))
                            Text(container.baseURL).font(.system(size: 12)).foregroundColor(
                                Color(.systemGray))
                        }
                    }

                    if let gitReleaseUrl = version.gitReleaseUrl {
                        Link(destination: URL(string: gitReleaseUrl)!) {
                            Text(
                                LocalizedStringResource(
                                    "ios.settings.info.version.label",
                                    defaultValue: "Version: \(version.gitRelease ?? "unknown")",
                                    bundle: #bundle,
                                    comment:
                                        "Row label in the information sheet showing the app version."
                                )
                            )
                            .foregroundColor(Color(.label))
                        }
                    } else {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.version.label",
                                defaultValue: "Version: \(version.gitRelease ?? "unknown")",
                                bundle: #bundle,
                                comment:
                                    "Row label in the information sheet showing the app version."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }

                    if let gitRevisionUrl = version.gitRevisionUrl {
                        Link(destination: URL(string: gitRevisionUrl)!) {
                            Text(
                                LocalizedStringResource(
                                    "ios.settings.info.git_revision.label",
                                    defaultValue:
                                        "Git revision: \(version.gitRevision ?? "unknown")",
                                    bundle: #bundle,
                                    comment:
                                        "Row label in the information sheet showing the git revision."
                                )
                            )
                            .foregroundColor(Color(.label))
                        }
                    } else {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.git_revision.label",
                                defaultValue:
                                    "Git revision: \(version.gitRevision ?? "unknown")",
                                bundle: #bundle,
                                comment:
                                    "Row label in the information sheet showing the git revision."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }

                    Link(destination: URL(string: "\(container.baseURL)/legal/tos")!) {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.terms_of_service.label",
                                defaultValue: "Terms of Service",
                                bundle: #bundle,
                                comment:
                                    "Link title in the information sheet that opens the Terms of Service page."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }

                    Link(destination: URL(string: "\(container.baseURL)/legal/privacy")!) {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.privacy_policy.label",
                                defaultValue: "Privacy Policy",
                                bundle: #bundle,
                                comment:
                                    "Link title in the information sheet that opens the Privacy Policy page."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }

                    Link(destination: URL(string: "https://koofr.eu/help/koofr-vault/")!) {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.help.label",
                                defaultValue: "Help",
                                bundle: #bundle,
                                comment:
                                    "Link title in the information sheet that opens the Koofr Vault help pages."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }

                    Link(destination: reportABugURL()) {
                        Text(
                            LocalizedStringResource(
                                "ios.settings.info.report_bug.label",
                                defaultValue: "Report a bug",
                                bundle: #bundle,
                                comment:
                                    "Link title in the information sheet that opens an email draft for reporting a bug."
                            )
                        )
                        .foregroundColor(Color(.label))
                    }
                }
            }
            .navigationTitle(
                Text(
                    LocalizedStringResource(
                        "ios.settings.info.title",
                        defaultValue: "Information",
                        bundle: #bundle,
                        comment: "Navigation title of the information sheet."
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
                                "ios.settings.info.done.button",
                                defaultValue: "Done",
                                bundle: #bundle,
                                comment: "Toolbar button that dismisses the information sheet."
                            )
                        )
                        .bold()
                    }
                }
            }
        }
    }

    private func reportABugURL() -> URL {
        let address = "support@koofr.net"
        let subject = "I Found A Bug in Vault iOS app"

        let body =
            "\n\nApp Version: \(version.gitRelease ?? "unknown")\nInternal device identifier: \(deviceModelIdentifier())\n\(UIDevice.current.systemName): \(UIDevice.current.systemVersion)"

        var components = URLComponents()
        components.scheme = "mailto"
        components.path = address
        components.queryItems = [
            URLQueryItem(name: "subject", value: subject),
            URLQueryItem(name: "body", value: body),
        ]

        return components.url!
    }

    private func deviceModelIdentifier() -> String {
        var systemInfo = utsname()
        uname(&systemInfo)
        let machineMirror = Mirror(reflecting: systemInfo.machine)
        let identifier = machineMirror.children.reduce("") { identifier, element in
            guard let value = element.value as? Int8, value != 0 else { return identifier }
            return identifier + String(UnicodeScalar(UInt8(value)))
        }
        return identifier
    }
}
