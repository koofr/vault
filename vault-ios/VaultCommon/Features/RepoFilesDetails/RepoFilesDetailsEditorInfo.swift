import SwiftUI
import VaultMobile

struct RepoFilesDetailsEditorInfo: View {
    var vm: RepoFilesDetailsScreenViewModel
    var info: RepoFilesDetailsInfo

    @ObservedObject var modifiedRelativeTime: RelativeTimeHelper

    public init(vm: RepoFilesDetailsScreenViewModel, info: RepoFilesDetailsInfo) {
        self.vm = vm
        self.info = info

        modifiedRelativeTime = RelativeTimeHelper(
            mobileVault: vm.container.mobileVault, value: info.fileModified)
    }

    var body: some View {
        VStack {
            if isLoading {
                Text(
                    LocalizedStringResource(
                        "ios.repo_files_details.loading.label",
                        defaultValue: "Loading…",
                        bundle: #bundle,
                        comment: "Status text above the text editor while file content is loading."
                    )
                )
                .font(.footnote)
                .foregroundStyle(.secondary)

            } else if isSaving {
                Text(
                    LocalizedStringResource(
                        "ios.repo_files_details.saving.label",
                        defaultValue: "Saving…",
                        bundle: #bundle,
                        comment:
                            "Status text above the text editor while file changes are being saved."
                    )
                )
                .font(.footnote)
                .foregroundStyle(.secondary)
            } else if let error = info.error {
                Text(error)
                    .font(.footnote)
                    .foregroundColor(.red)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal)
                    .accessibilityIdentifier("File error")
            } else {
                HStack(alignment: .center, spacing: 15) {
                    VStack(spacing: 2) {
                        Text(
                            LocalizedStringResource(
                                "ios.repo_files_details.auto_save_info.label",
                                defaultValue: "Changes are saved automatically.",
                                bundle: #bundle,
                                comment:
                                    "Informational text above the text editor indicating autosave behavior."
                            )
                        )
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        if let modified = modifiedRelativeTime.display {
                            Text(
                                LocalizedStringResource(
                                    "ios.repo_files_details.last_saved.label",
                                    defaultValue: "Last saved \(modified)",
                                    bundle: #bundle,
                                    comment:
                                        "Autosave status line above the editor showing relative time of last save."
                                )
                            )
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        }
                    }

                    Circle()
                        .fill(info.isDirty ? .orange : .green)
                        .frame(width: 8, height: 8)
                        .accessibilityLabel(
                            Text(
                                info.isDirty
                                    ? LocalizedStringResource(
                                        "ios.repo_files_details.status.dirty.a11y.label",
                                        defaultValue: "File modified",
                                        bundle: #bundle,
                                        comment:
                                            "Accessibility label for editor status indicator when there are unsaved file changes."
                                    )
                                    : LocalizedStringResource(
                                        "ios.repo_files_details.status.unchanged.a11y.label",
                                        defaultValue: "File unchanged",
                                        bundle: #bundle,
                                        comment:
                                            "Accessibility label for editor status indicator when file content is unchanged."
                                    )
                            )
                        )
                }
            }
        }
        .frame(maxWidth: .infinity, minHeight: minHeight)
        .task {
            await modifiedRelativeTime.updateLoop()
        }
    }

    private var isLoading: Bool {
        if case .loading = info.status { return true }
        if case .loading = info.contentStatus { return true }
        return false
    }

    private var isSaving: Bool {
        if case .loading = info.saveStatus { return true }
        return false
    }

    private var minHeight: CGFloat {
        let lineHeight = UIFontMetrics(forTextStyle: .footnote)
            .scaledFont(for: UIFont.preferredFont(forTextStyle: .footnote))
            .lineHeight

        let spacing: CGFloat = 2
        let lines: CGFloat = 2

        return (lineHeight * lines) + spacing
    }
}
