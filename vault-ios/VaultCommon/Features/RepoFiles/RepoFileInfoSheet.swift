import SwiftUI
import VaultMobile

struct RepoFileInfoSheet: View {
    public let vm: RepoFilesScreenViewModel
    public let file: RepoFile
    public let onDismiss: () -> Void

    @ObservedObject var modifiedRelativeTime: RelativeTimeHelper

    var categoryDisplay: String {
        switch file.category {
        case .generic: return "File"
        case .folder: return "Folder"
        case .archive: return "Archive"
        case .audio: return "Audio"
        case .code: return "Code"
        case .document: return "Document"
        case .image: return "Image"
        case .pdf: return "PDF"
        case .presentation: return "Presentation"
        case .sheet: return "Spreadsheet"
        case .text: return "Text"
        case .video: return "Video"
        }
    }

    init(vm: RepoFilesScreenViewModel, file: RepoFile, onDismiss: @escaping () -> Void) {
        self.vm = vm
        self.file = file
        self.onDismiss = onDismiss

        self.modifiedRelativeTime = RelativeTimeHelper(
            mobileVault: vm.container.mobileVault, value: file.modified)
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

                    Text("Information").font(.headline).padding(.bottom, 20)

                    HStack {
                        Text("Type").font(.system(size: 15)).foregroundColor(Color(.systemGray2))
                        Spacer()
                        Text(categoryDisplay).font(.system(size: 15))
                    }
                    .padding(.bottom, 10)

                    Divider().padding(.bottom, 10)

                    if !file.sizeDisplay.isEmpty {
                        HStack {
                            Text("Size").font(.system(size: 15)).foregroundColor(
                                Color(.systemGray2))
                            Spacer()
                            Text(file.sizeDisplay).font(.system(size: 15))
                        }
                        .padding(.bottom, 10)

                        Divider().padding(.bottom, 10)
                    }

                    if let modified = file.modified {
                        if let modifiedRelativeTimeDisplay = modifiedRelativeTime.display {
                            HStack(alignment: .top) {
                                Text("Modified").font(.system(size: 15)).foregroundColor(
                                    Color(.systemGray2))
                                Spacer()
                                VStack(alignment: .trailing, spacing: 10) {
                                    Text(modifiedRelativeTimeDisplay).font(.system(size: 15))
                                    Text(
                                        Date(timeIntervalSince1970: Double(modified) / 1000)
                                            .formatted(
                                                date: .long, time: .standard)
                                    ).font(.system(size: 15))
                                }
                            }
                            .padding(.bottom, 10)

                            Divider().padding(.bottom, 10)
                        }
                    }

                    HStack {
                        Text("Path").font(.system(size: 15)).foregroundColor(Color(.systemGray2))
                        Spacer()
                        Text(file.decryptedPath ?? "???")
                            .font(.system(size: 15))
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(.bottom, 10)

                    Divider().padding(.bottom, 10)

                    HStack {
                        Text("Encrypted path").font(.system(size: 15)).foregroundColor(
                            Color(.systemGray2))
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
                        Button("Done") {
                            onDismiss()
                        }
                    }
                }
                .navigationTitle("Info")
                .navigationBarTitleDisplayMode(.inline)
            }
        }
        .task {
            await modifiedRelativeTime.updateLoop()
        }
    }
}
