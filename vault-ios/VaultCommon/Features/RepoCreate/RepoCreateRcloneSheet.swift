import SwiftUI
import VaultMobile

struct RepoCreateRcloneSheet: View {
    public enum Field {
        case config
    }

    public let vm: RepoCreateScreenViewModel
    public let onDismiss: () -> Void

    @ObservedObject private var info: Subscription<RepoCreateInfo>

    @State var config: String = ""
    @FocusState var focusedField: Field?

    init(vm: RepoCreateScreenViewModel, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.onDismiss = onDismiss

        info = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.repoCreateInfoSubscribe(createId: vm.createId, cb: cb)
            },
            getData: { v, id in
                v.repoCreateInfoData(id: id)
            })
    }

    var body: some View {
        NavigationView {
            VStack {
                if let info = info.data {
                    switch info {
                    case .form(let form):
                        if let error = form.fillFromRcloneConfigError {
                            HStack {
                                Text(error)
                                    .font(.body)
                                    .foregroundColor(Color(.systemRed))
                                    .padding(.bottom, 10)

                                Spacer()
                            }
                        }
                    default: EmptyView()
                    }
                }

                let rcloneExample = """
                    [name]
                    type=crypt
                    remote=rcloneremote:/path
                    password=obscured password
                    password2=obscured salt
                    """

                TextField(
                    LocalizedStringResource(
                        "ios.repo_create_rclone.rclone_config.placeholder",
                        defaultValue: """
                            rclone config

                            Format:

                            \(rcloneExample)
                            """,
                        bundle: #bundle,
                        comment:
                            "Placeholder text for the multiline rclone config input field in the import sheet."
                    ),
                    text: $config,
                    axis: .vertical
                )
                .font(.system(.body, design: .monospaced))
                .lineLimit(9...)
                .padding(.bottom, 20)
                .onAppear {
                    focusedField = .config
                }
                .accessibilityLabel(
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create_rclone.rclone_config.a11y.label",
                            defaultValue: "rclone config",
                            bundle: #bundle,
                            comment: "Accessibility label for the rclone config text input field."
                        )
                    )
                )

                HStack {
                    PasteButton(payloadType: String.self) { strings in
                        config = strings[0]
                    }

                    Spacer()
                }

                Spacer()
            }
            .padding()
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        onDismiss()
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create_rclone.cancel.button",
                                defaultValue: "Cancel",
                                bundle: #bundle,
                                comment:
                                    "Toolbar button that closes the rclone config import sheet without applying changes."
                            )
                        )
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        if vm.fillFromRcloneConfig(config: config) {
                            onDismiss()
                        }
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create_rclone.fill.button",
                                defaultValue: "Fill",
                                bundle: #bundle,
                                comment:
                                    "Toolbar button that parses the pasted rclone config and fills the create Safe Box form."
                            )
                        )
                    }
                }
            }
            .navigationTitle(
                Text(
                    LocalizedStringResource(
                        "ios.repo_create_rclone.title",
                        defaultValue: "From rclone config",
                        bundle: #bundle,
                        comment:
                            "Navigation title for the sheet that imports Safe Box settings from an rclone config."
                    )
                )
            )
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}
