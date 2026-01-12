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
                Text("Loading...")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

            } else if isSaving {
                Text("Saving...")
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
                        Text("Changes are saved automatically.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        if let modified = modifiedRelativeTime.display {
                            Text("Last saved \(modified)")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                    }

                    Circle()
                        .fill(info.isDirty ? .orange : .green)
                        .frame(width: 8, height: 8)
                        .accessibilityLabel(info.isDirty ? "File modified" : "File unchanged")
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
