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

                TextField(
                    """
                    rclone config

                    Format:

                    [name]
                    type=crypt
                    remote=rcloneremote:/path
                    password=obscured password
                    password2=obscured salt
                    """, text: $config, axis: .vertical
                )
                .font(.system(.body, design: .monospaced))
                .lineLimit(9...)
                .padding(.bottom, 20)
                .onAppear {
                    focusedField = .config
                }
                .accessibilityLabel("rclone config")

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
                    Button("Cancel") {
                        onDismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button("Fill") {
                        if vm.fillFromRcloneConfig(config: config) {
                            onDismiss()
                        }
                    }
                }
            }
            .navigationBarTitle("Fill from rclone config", displayMode: .inline)
        }
    }
}
