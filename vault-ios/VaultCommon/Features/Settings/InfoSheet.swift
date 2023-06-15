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
                            Text("Koofr Vault").foregroundColor(Color(.label))
                            Text(container.baseURL).font(.system(size: 12)).foregroundColor(
                                Color(.systemGray))
                        }
                    }

                    if let gitReleaseUrl = version.gitReleaseUrl {
                        Link(destination: URL(string: gitReleaseUrl)!) {
                            Text("Version: \(version.gitRelease ?? "unknown")").foregroundColor(
                                Color(.label))
                        }
                    } else {
                        Text("Version: \(version.gitRelease ?? "unknown")").foregroundColor(
                            Color(.label))
                    }

                    if let gitRevisionUrl = version.gitRevisionUrl {
                        Link(destination: URL(string: gitRevisionUrl)!) {
                            Text("Git revision: \(version.gitRevision ?? "unknown")")
                                .foregroundColor(Color(.label))
                        }
                    } else {
                        Text("Git revision: \(version.gitRevision ?? "unknown")").foregroundColor(
                            Color(.label))
                    }

                    Link(destination: URL(string: "\(container.baseURL)/legal/tos")!) {
                        Text("Terms of Service").foregroundColor(Color(.label))
                    }

                    Link(destination: URL(string: "\(container.baseURL)/legal/privacy")!) {
                        Text("Privacy Policy").foregroundColor(Color(.label))
                    }

                    Link(destination: URL(string: "https://koofr.eu/help/koofr-vault/")!) {
                        Text("Help").foregroundColor(Color(.label))
                    }

                    Link(destination: reportABugURL()) {
                        Text("Report a bug").foregroundColor(Color(.label))
                    }
                }
            }
            .navigationTitle("Information")
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
