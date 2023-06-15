import SwiftUI
import UniformTypeIdentifiers
import VaultMobile

public struct RepoConfigInfo: View {
    public let config: RepoConfig
    public let onSave: () -> Void

    func getInfo() -> AttributedString {
        let normal: (String) -> AttributedString = { string in
            var attributedString = AttributedString(string)
            attributedString.font = .body
            return attributedString
        }
        let bold: (String) -> AttributedString = { string in
            var attributedString = AttributedString(string)
            attributedString.font = .body.bold()
            return attributedString
        }
        let monospaced: (String) -> AttributedString = { string in
            var attributedString = AttributedString(string)
            attributedString.font = .body.monospaced()
            return attributedString
        }

        var info = AttributedString()

        info.append(bold("Location: "))
        info.append(normal(config.location.path))
        info.append(normal("\n\n"))

        info.append(bold("Filename encryption: "))
        info.append(normal("standard"))
        info.append(normal("\n\n"))

        info.append(bold("Encrypt directory names: "))
        info.append(normal("true"))
        info.append(normal("\n\n"))

        info.append(bold("Safe Key (password): "))
        info.append(normal(config.password))
        info.append(normal("\n\n"))

        info.append(bold("Salt (password2): "))
        info.append(normal(config.salt ?? ""))
        info.append(normal("\n\n"))

        info.append(bold("rclone config: "))
        info.append(normal("\n\n"))

        info.append(monospaced(config.rcloneConfig))

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
