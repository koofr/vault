import SwiftUI
import VaultMobile

struct RepoFileInfoSheet: View {
    public let vm: RepoFilesScreenViewModel
    public let file: RepoFile
    public let onDismiss: () -> Void

    @ObservedObject var modifiedRelativeTime: RelativeTimeHelper

    @ObservedObject var repoInfo: Subscription<RepoInfo>

    @Environment(\.locale) private var locale

    var categoryDisplay: LocalizedStringResource {
        switch file.category {
        case .generic:
            return LocalizedStringResource(
                "ios.repo_file_category.file",
                defaultValue: "File",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for generic files."
            )
        case .folder:
            return LocalizedStringResource(
                "ios.repo_file_category.folder",
                defaultValue: "Folder",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for folders."
            )
        case .archive:
            return LocalizedStringResource(
                "ios.repo_file_category.archive",
                defaultValue: "Archive",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for archive files."
            )
        case .audio:
            return LocalizedStringResource(
                "ios.repo_file_category.audio",
                defaultValue: "Audio",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for audio files."
            )
        case .code:
            return LocalizedStringResource(
                "ios.repo_file_category.code",
                defaultValue: "Code",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for code files."
            )
        case .document:
            return LocalizedStringResource(
                "ios.repo_file_category.document",
                defaultValue: "Document",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for document files."
            )
        case .image:
            return LocalizedStringResource(
                "ios.repo_file_category.image",
                defaultValue: "Image",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for image files."
            )
        case .pdf:
            return LocalizedStringResource(
                "ios.repo_file_category.pdf",
                defaultValue: "PDF",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for PDF files."
            )
        case .presentation:
            return LocalizedStringResource(
                "ios.repo_file_category.presentation",
                defaultValue: "Presentation",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for presentation files."
            )
        case .sheet:
            return LocalizedStringResource(
                "ios.repo_file_category.sheet",
                defaultValue: "Spreadsheet",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for spreadsheet files."
            )
        case .text:
            return LocalizedStringResource(
                "ios.repo_file_category.text",
                defaultValue: "Text",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for plain text files."
            )
        case .video:
            return LocalizedStringResource(
                "ios.repo_file_category.video",
                defaultValue: "Video",
                bundle: #bundle,
                comment: "File category label shown in file info sheet for video files."
            )
        }
    }

    init(vm: RepoFilesScreenViewModel, file: RepoFile, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.file = file
        self.onDismiss = onDismiss

        self.modifiedRelativeTime = RelativeTimeHelper(
            mobileVault: vm.container.mobileVault, value: file.modified)

        self.repoInfo = Subscription(
            mobileVault: vm.container.mobileVault,
            subscribe: { v, cb in
                v.reposRepoSubscribe(repoId: file.repoId, cb: cb)
            },
            getData: { v, id in
                v.reposRepoData(id: id)
            })

        self.repoInfo.setOnData { data in
            if let repo = data?.repo {
                if repo.state == .locked {
                    // dismiss the repo file info sheet when repo is locked
                    onDismiss()
                }
            }
        }

    }

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    HStack {
                        Spacer()

                        FileIcon(
                            fileIconCache: vm.container.fileIconCache, attrs: file.fileIconAttrs,
                            size: .lg, scale: 4, height: 136)

                        Spacer()
                    }
                    .padding(.top, 50)
                    .padding(.bottom, 50)

                    if let nameError = file.nameError {
                        Text(file.name).font(.title2).foregroundColor(Color(.systemRed)).padding(
                            .bottom, 5)

                        Text(nameError).font(.system(size: 15)).foregroundColor(Color(.systemRed))
                            .padding(
                                .bottom, 20)
                    } else {
                        Text(file.name).font(.title2).padding(.bottom, 20)
                    }

                    Text(
                        LocalizedStringResource(
                            "ios.repo_file_info.information.label",
                            defaultValue: "Information",
                            bundle: #bundle,
                            comment: "Section header title in file info sheet."
                        )
                    )
                    .font(.headline)
                    .padding(.bottom, 20)

                    HStack {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_file_info.type.label",
                                defaultValue: "Type",
                                bundle: #bundle,
                                comment: "Field label in file info sheet for file category/type."
                            )
                        )
                        .font(.system(size: 15))
                        .foregroundColor(Color(.systemGray2))
                        Spacer()
                        Text(categoryDisplay).font(.system(size: 15))
                    }
                    .padding(.bottom, 10)

                    Divider().padding(.bottom, 10)

                    if let sizeDisplay = file.sizeDisplay {
                        if !sizeDisplay.isEmpty {
                            HStack {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_file_info.size.label",
                                        defaultValue: "Size",
                                        bundle: #bundle,
                                        comment: "Field label in file info sheet for file size."
                                    )
                                )
                                .font(.system(size: 15))
                                .foregroundColor(Color(.systemGray2))
                                Spacer()
                                Text(sizeDisplay).font(.system(size: 15))
                            }
                            .padding(.bottom, 10)

                            Divider().padding(.bottom, 10)
                        }
                    }

                    if let modified = file.modified {
                        if let modifiedRelativeTimeDisplay = modifiedRelativeTime.display {
                            HStack(alignment: .top) {
                                Text(
                                    LocalizedStringResource(
                                        "ios.repo_file_info.modified.label",
                                        defaultValue: "Modified",
                                        bundle: #bundle,
                                        comment:
                                            "Field label in file info sheet for last modification date."
                                    )
                                )
                                .font(.system(size: 15))
                                .foregroundColor(Color(.systemGray2))
                                Spacer()
                                VStack(alignment: .trailing, spacing: 10) {
                                    Text(modifiedRelativeTimeDisplay).font(.system(size: 15))
                                    Text(
                                        Date(timeIntervalSince1970: Double(modified) / 1000)
                                            .formatted(
                                                Date.FormatStyle(
                                                    date: .long, time: .standard, locale: locale))
                                    ).font(.system(size: 15))
                                }
                            }
                            .padding(.bottom, 10)

                            Divider().padding(.bottom, 10)
                        }
                    }

                    HStack {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_file_info.path.label",
                                defaultValue: "Path",
                                bundle: #bundle,
                                comment: "Field label in file info sheet for decrypted path."
                            )
                        )
                        .font(.system(size: 15))
                        .foregroundColor(Color(.systemGray2))
                        Spacer()
                        Text(file.decryptedPath ?? "???")
                            .font(.system(size: 15))
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(.bottom, 10)

                    Divider().padding(.bottom, 10)

                    HStack {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_file_info.encrypted_path.label",
                                defaultValue: "Encrypted path",
                                bundle: #bundle,
                                comment: "Field label in file info sheet for encrypted path."
                            )
                        )
                        .font(.system(size: 15))
                        .foregroundColor(Color(.systemGray2))
                        Spacer()
                        Text(file.encryptedPath)
                            .font(.system(size: 15))
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(.bottom, 10)

                    Spacer()
                }
                .padding()
                .toolbar {
                    ToolbarItem(placement: .confirmationAction) {
                        Button {
                            onDismiss()
                        } label: {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_file_info.done.button",
                                    defaultValue: "Done",
                                    bundle: #bundle,
                                    comment: "Toolbar button that closes the file info sheet."
                                )
                            )
                        }
                    }
                }
                .navigationTitle(
                    Text(
                        LocalizedStringResource(
                            "ios.repo_file_info.title",
                            defaultValue: "Info",
                            bundle: #bundle,
                            comment: "Navigation title of the file info sheet."
                        )
                    )
                )
                .navigationBarTitleDisplayMode(.inline)
            }
        }
        .task {
            await modifiedRelativeTime.updateLoop()
        }
    }
}
