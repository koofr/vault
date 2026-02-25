import SwiftUI
import UniformTypeIdentifiers
import VaultMobile
import VaultUtils

public struct RepoConfigInfo: View {
    public let config: RepoConfig
    public let onSave: () -> Void

    @Environment(\.locale) private var locale

    func getInfo() -> AttributedString {
        var info = AttributedString()
        info.font = .body

        let locationPlaceholder = "LOCATION_PLACEHOLDER"
        info.append(
            AttributedString.markdownLocalized(
                LocalizedStringResource(
                    "ios.repo_config_info.location",
                    defaultValue: "**Location**: \(locationPlaceholder)",
                    locale: locale,
                    bundle: #bundle,
                    comment:
                        "Text in the Safe Box configuration summary indicating storage location."
                )
            ) { attributedString in
                attributedString.replaceLiteralToken(
                    locationPlaceholder, with: config.location.path)
            })
        info.append(AttributedString("\n\n"))

        let filenameEncryption = "standard"
        info.append(
            AttributedString(
                localized: LocalizedStringResource(
                    "ios.repo_config_info.filename_encryption",
                    defaultValue: "**Filename encryption**: \(filenameEncryption)",
                    locale: locale,
                    bundle: #bundle,
                    comment:
                        "Text in the Safe Box configuration summary for filename encryption mode."
                )
            )
        )
        info.append(AttributedString("\n\n"))

        let encryptDirectoryNames = "true"
        info.append(
            AttributedString(
                localized: LocalizedStringResource(
                    "ios.repo_config_info.encrypt_directory_names",
                    defaultValue: "**Encrypt directory names**: \(encryptDirectoryNames)",
                    locale: locale,
                    bundle: #bundle,
                    comment:
                        "Text in the Safe Box configuration summary for directory name encryption setting."
                )

            )
        )
        info.append(AttributedString("\n\n"))

        let saltPlaceholder = "SALT_PLACEHOLDER"
        info.append(
            AttributedString.markdownLocalized(
                LocalizedStringResource(
                    "ios.repo_config_info.salt",
                    defaultValue: "**Salt (password2)**: \(saltPlaceholder)",
                    locale: locale,
                    bundle: #bundle,
                    comment:
                        "Text in the Safe Box configuration summary for the salt (password2) value."
                )
            ) { attributedString in
                attributedString.replaceLiteralToken(saltPlaceholder, with: config.salt ?? "")
            })
        info.append(AttributedString("\n\n"))

        info.append(
            AttributedString(
                localized: LocalizedStringResource(
                    "ios.repo_config_info.rclone_config",
                    defaultValue: "**rclone config**:",
                    locale: locale,
                    bundle: #bundle,
                    comment:
                        "Text before the raw rclone configuration block in the Safe Box setup summary."
                )
            )
        )
        info.append(AttributedString("\n\n"))

        var rcloneConfig = AttributedString(config.rcloneConfig)
        rcloneConfig.font = .body.monospaced()
        info.append(rcloneConfig)

        return info
    }

    public var body: some View {
        let info = getInfo()
        let infoText = String(info.characters)

        VStack {
            HStack {
                Text(info).textSelection(.enabled)

                Spacer()
            }

            HStack {
                ShareLink(item: infoText)
                    .buttonStyle(.borderedProminent)
                    .simultaneousGesture(
                        TapGesture().onEnded {
                            onSave()
                        })

                Spacer()
            }
        }
    }
}

public struct RepoConfigInfo_Previews: PreviewProvider {
    static public var previews: some View {
        VStack {
            RepoConfigInfo(config: PreviewsData.repoConfig, onSave: {})
            Spacer()
        }
        .padding()
    }
}
