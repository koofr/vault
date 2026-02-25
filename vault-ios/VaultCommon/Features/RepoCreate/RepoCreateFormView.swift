import SwiftUI
import VaultMobile

private let repoCreateFormLocationInfoText = LocalizedStringResource(
    "ios.repo_create.form.location.info.text",
    defaultValue: """
        Location refers to a folder within your Koofr where all of your Safe Box files and folders are securely stored.

        If this is your first Safe Box, the default location will be "My safe box." You can change it if you prefer.

        If you already have a Safe Box or wish to use an existing folder (e.g. one created with rclone), you can select that folder.

        Please note that you can only select a folder located within your Koofr.
        """,
    bundle: #bundle,
    comment: "Detailed help text in create Safe Box flow explaining the Location field."
)

private let repoCreateFormSafeKeyInfoText = LocalizedStringResource(
    "ios.repo_create.form.password.info.text",
    defaultValue: """
        Safe Key is a password used to encrypt your files. Each Safe Box can have its own unique Safe Key.

        Please be aware that once you set your Safe Key, it cannot be changed later. All the files within the Safe Box will be encrypted using this key.

        IMPORTANT: Your Safe Key cannot be reset, and there is no way to recover your files if you forget it, as it is never sent to or stored on Koofr servers.
        """,
    bundle: #bundle,
    comment:
        "Detailed help text in create Safe Box flow explaining the Safe Key field and recovery limitations."
)

private let repoCreateFormSaltInfoText = LocalizedStringResource(
    "ios.repo_create.form.salt.info.text",
    defaultValue: """
        Salt is used in the key derivation process to create a unique encryption key and helps to protect against potential attacks. It will be stored on the Koofr servers in a secure manner.

        A random Salt has been generated for you. If you prefer, you can leave the Salt field empty, and the default salt will be used (same as in rclone). However, it is recommended to use a unique salt for enhanced security. Using a unique salt helps to increase the complexity of the encryption process, making it more difficult for potential attackers to access the encrypted data.

        If you wish to transfer the encrypted files to another service, it is necessary to also export the salt, otherwise you won't be able to decrypt your files.
        """,
    bundle: #bundle,
    comment: "Detailed help text in create Safe Box flow explaining the Salt field."
)

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
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.form.location.label",
                            defaultValue: "Location",
                            bundle: #bundle,
                            comment:
                                "Section header label for storage location in create Safe Box form."
                        )
                    )
                    Spacer()
                    Button {
                        vm.container.sheets.show(name: "repoCreateLocationInfo") { _, hide in
                            FormInfoSheet(
                                title: LocalizedStringResource(
                                    "ios.repo_create.form.location.info.title",
                                    defaultValue: "Location",
                                    bundle: #bundle,
                                    comment:
                                        "Title of the location help sheet in create Safe Box form."
                                ),
                                text: repoCreateFormLocationInfoText,
                                onDismiss: hide)
                        }
                    } label: {
                        Image(systemName: "questionmark.circle")
                    }
                    .tint(Color(.systemGray))
                    .accessibilityLabel(
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.location.info.button.a11y.label",
                                defaultValue: "Location info",
                                bundle: #bundle,
                                comment:
                                    "Accessibility label for the location help button in create Safe Box form."
                            )
                        )
                    )
                }
            ) {
                HStack {
                    if form.locationBreadcrumbs.isEmpty {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.location.placeholder",
                                defaultValue: "Location",
                                bundle: #bundle,
                                comment:
                                    "Placeholder shown when no location is selected in create Safe Box form."
                            )
                        )
                        .foregroundColor(Color(.systemGray3))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .contentShape(Rectangle())
                    } else {
                        RemoteFilesBreadcrumbs(breadcrumbs: form.locationBreadcrumbs)
                    }
                }
                .frame(maxWidth: .infinity)
                .accessibilityLabel(
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.form.location.a11y.label",
                            defaultValue: "Location",
                            bundle: #bundle,
                            comment:
                                "Accessibility label for the selected location row in create Safe Box form."
                        )
                    )
                )
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
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.form.password.label",
                            defaultValue: "Safe Key",
                            bundle: #bundle,
                            comment:
                                "Section header label for safe key input in create Safe Box form."
                        )
                    )
                    Spacer()
                    Button {
                        vm.container.sheets.show(name: "repoCreateSafeKeyInfo") { _, hide in
                            FormInfoSheet(
                                title: LocalizedStringResource(
                                    "ios.repo_create.form.password.info.title",
                                    defaultValue: "Safe Key",
                                    bundle: #bundle,
                                    comment:
                                        "Title of the safe key help sheet in create Safe Box form."
                                ),
                                text: repoCreateFormSafeKeyInfoText,
                                onDismiss: hide)
                        }
                    } label: {
                        Image(systemName: "questionmark.circle")
                    }
                    .tint(Color(.systemGray))
                    .accessibilityLabel(
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.password.info.button.a11y.label",
                                defaultValue: "Safe Key info",
                                bundle: #bundle,
                                comment:
                                    "Accessibility label for the safe key help button in create Safe Box form."
                            )
                        )
                    )
                }
            ) {
                RepoPasswordField(
                    text: password,
                    inline: true,
                    label: LocalizedStringResource(
                        "ios.repo_create.form.password.placeholder",
                        defaultValue: "Must be at least 8 characters long",
                        bundle: #bundle,
                        comment:
                            "Placeholder for the Safe Key field describing minimum length requirement."
                    )
                )
            }

            if advancedVisible {
                Section(
                    header: HStack {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.salt.label",
                                defaultValue: "Salt",
                                bundle: #bundle,
                                comment:
                                    "Section header label for optional salt input in advanced create settings."
                            )
                        )
                        Spacer()
                        Button {
                            vm.container.sheets.show(name: "repoCreateSaltInfo") { _, hide in
                                FormInfoSheet(
                                    title: LocalizedStringResource(
                                        "ios.repo_create.form.salt.info.title",
                                        defaultValue: "Salt",
                                        bundle: #bundle,
                                        comment:
                                            "Title of the salt help sheet in create Safe Box form."
                                    ),
                                    text: repoCreateFormSaltInfoText,
                                    onDismiss: hide
                                )
                            }
                        } label: {
                            Image(systemName: "questionmark.circle")
                        }
                        .tint(Color(.systemGray))
                        .accessibilityLabel(
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_create.form.salt.info.button.a11y.label",
                                    defaultValue: "Salt info",
                                    bundle: #bundle,
                                    comment:
                                        "Accessibility label for the salt help button in create Safe Box form."
                                )
                            )
                        )
                    }
                ) {
                    TextField(
                        LocalizedStringResource(
                            "ios.repo_create.form.salt.placeholder",
                            defaultValue: "Salt",
                            bundle: #bundle,
                            comment:
                                "Placeholder for optional salt text field in advanced create Safe Box settings."
                        ),
                        text: salt,
                        axis: .vertical
                    )
                    .textInputAutocapitalization(.never)
                    .keyboardType(.asciiCapable)
                    .autocorrectionDisabled()
                    .accessibilityLabel(
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.salt.a11y.label",
                                defaultValue: "Salt",
                                bundle: #bundle,
                                comment:
                                    "Accessibility label for the salt input field in create Safe Box form."
                            )
                        )
                    )
                }

                Section {
                    Button {
                        vm.container.sheets.show(name: "repoCreateRclone") { _, hide in
                            RepoCreateRcloneSheet(vm: vm, onDismiss: hide)
                        }
                    } label: {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.from_rclone_config.button",
                                defaultValue: "From rclone config",
                                bundle: #bundle,
                                comment:
                                    "Button in advanced create Safe Box settings that opens fill from rclone config sheet."
                            )
                        )
                        .frame(maxWidth: .infinity)
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
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.show_advanced_settings.button",
                                defaultValue: "Show advanced settings",
                                bundle: #bundle,
                                comment:
                                    "Button in create Safe Box form that reveals advanced fields such as salt and fill from rclone config."
                            )
                        )
                        .frame(maxWidth: .infinity)
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
                        Text(
                            LocalizedStringResource(
                                "ios.repo_create.form.create.button",
                                defaultValue: "Create",
                                bundle: #bundle,
                                comment: "Toolbar confirmation button that creates the Safe Box."
                            )
                        )
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
