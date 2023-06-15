import SwiftUI
import VaultMobile

private let repoCreateFormLocationInfoText = """
    Location refers to a folder within your Koofr where all of your Safe Box files and folders are securely stored.

    If this is your first Safe Box, the default location will be "My safe box." You can change it if you prefer.

    If you already have a Safe Box or wish to use an existing folder (e.g. one created with rclone), you can select that folder.

    Please note that you can only select a folder located within your Koofr.
    """

private let repoCreateFormSafeKeyInfoText = """
    Safe Key is a password used to encrypt your files. Each Safe Box can have its own unique Safe Key.

    Please be aware that once you set your Safe Key, it cannot be changed later. All the files within the Safe Box will be encrypted using this key.

    IMPORTANT: Your Safe Key cannot be reset, and there is no way to recover your files if you forget it, as it is never sent to or stored on Koofr servers.
    """

private let repoCreateFormSaltInfoText = """
    Salt is used in the key derivation process to create a unique encryption key and helps to protect against potential attacks. It will be stored on the Koofr servers in a secure manner.

    A random Salt has been generated for you. If you prefer, you can leave the Salt field empty, and the default salt will be used (same as in rclone). However, it is recommended to use a unique salt for enhanced security. Using a unique salt helps to increase the complexity of the encryption process, making it more difficult for potential attackers to access the encrypted data.

    If you wish to transfer the encrypted files to another service, it is necessary to also export the salt, otherwise you won't be able to decrypt your files.
    """

public struct RepoCreateFormView: View {
    @ObservedObject public var vm: RepoCreateScreenViewModel
    public let form: RepoCreateForm

    @State var advancedVisible = false

    public var canCreate: Bool {
        if !form.canCreate {
            return false
        }

        switch form.createRepoStatus {
        case .loading: return false
        default: return true
        }
    }

    public init(vm: RepoCreateScreenViewModel, form: RepoCreateForm) {
        self.vm = vm
        self.form = form
    }

    public var body: some View {
        let password = Binding(
            get: {
                vm.password
            },
            set: { value in
                vm.setPassword(password: value)
            })

        let salt = Binding(
            get: {
                vm.salt
            },
            set: { value in
                vm.setSalt(salt: value)
            })

        Form {
            Section(
                header: HStack {
                    Text("Location")
                    Spacer()
                    Button {
                        vm.container.sheets.show(name: "repoCreateLocationInfo") { _, hide in
                            FormInfoSheet(
                                title: "Location", text: repoCreateFormLocationInfoText,
                                onDismiss: hide)
                        }
                    } label: {
                        Image(systemName: "questionmark.circle")
                    }
                    .tint(Color(.systemGray))
                    .accessibilityLabel("Location info")
                }
            ) {
                HStack {
                    if form.locationBreadcrumbs.isEmpty {
                        Text("Location")
                            .foregroundColor(Color(.systemGray3))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .contentShape(Rectangle())
                    } else {
                        RemoteFilesBreadcrumbs(breadcrumbs: form.locationBreadcrumbs)
                    }
                }
                .frame(maxWidth: .infinity)
                .accessibilityLabel("Location")
                .onTapGesture {
                    showLocationPicker()
                }

                switch form.createRepoStatus {
                case .err(let error, _):
                    Text(error)
                        .font(.body)
                        .foregroundColor(Color(.systemRed))
                        .frame(alignment: .leading)
                default:
                    EmptyView()
                }
            }

            Section(
                header: HStack {
                    Text("Safe Key")
                    Spacer()
                    Button {
                        vm.container.sheets.show(name: "repoCreateSafeKeyInfo") { _, hide in
                            FormInfoSheet(
                                title: "Safe Key", text: repoCreateFormSafeKeyInfoText,
                                onDismiss: hide)
                        }
                    } label: {
                        Image(systemName: "questionmark.circle")
                    }
                    .tint(Color(.systemGray))
                    .accessibilityLabel("Safe Key info")
                }
            ) {
                RepoPasswordField(
                    text: password, inline: true, label: "Must be at least 8 characters long")
            }

            if advancedVisible {
                Section(
                    header: HStack {
                        Text("Salt")
                        Spacer()
                        Button {
                            vm.container.sheets.show(name: "repoCreateSaltInfo") { _, hide in
                                FormInfoSheet(
                                    title: "Salt", text: repoCreateFormSaltInfoText, onDismiss: hide
                                )
                            }
                        } label: {
                            Image(systemName: "questionmark.circle")
                        }
                        .tint(Color(.systemGray))
                        .accessibilityLabel("Salt info")
                    }
                ) {
                    TextField("Salt", text: salt, axis: .vertical)
                        .textInputAutocapitalization(.never)
                        .keyboardType(.asciiCapable)
                        .autocorrectionDisabled()
                        .accessibilityLabel("Salt")
                }

                Section {
                    Button {
                        vm.container.sheets.show(name: "repoCreateRclone") { _, hide in
                            RepoCreateRcloneSheet(vm: vm, onDismiss: hide)
                        }
                    } label: {
                        Text("From rclone config").frame(maxWidth: .infinity)
                    }
                }
            } else {
                Section {
                    Button {
                        UIApplication.shared.sendAction(
                            #selector(UIResponder.resignFirstResponder), to: nil, from: nil,
                            for: nil)

                        advancedVisible = true
                    } label: {
                        Text("Show advanced settings").frame(maxWidth: .infinity)
                    }
                }
            }
        }
        .onSubmit {
            if canCreate {
                vm.create()
            }
        }
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button(
                    action: {
                        vm.create()
                    },
                    label: {
                        Text("Create")
                    }
                )
                .disabled(!canCreate)
            }
        }
    }

    func showLocationPicker() {
        vm.container.sheets.show(
            name: "repoCreateRemoteFilesDirPicker",
            viewModel: RemoteFilesDirPickerViewModel(
                container: vm.container,
                canSelect: { _, path in
                    path != "/"
                },
                onSelect: { mountId, path in
                    vm.setLocation(mountId: mountId, path: path)

                    vm.container.sheets.hide(name: "repoCreateRemoteFilesDirPicker")
                },
                onCancel: {
                    vm.container.sheets.hide(name: "repoCreateRemoteFilesDirPicker")
                }
            )
        ) { vm, hide in
            RemoteFilesDirPickerNavigation(vm: vm)
        }
    }
}
