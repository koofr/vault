import SwiftUI
import VaultMobile

public struct RepoCreateCreatedView: View {
    public let config: RepoConfig
    public let onContinue: () -> Void

    @State private var saved: Bool = false

    init(config: RepoConfig, onContinue: @escaping () -> Void) {
        self.config = config
        self.onContinue = onContinue
    }

    public var body: some View {
        ScrollView {
            VStack {
                HStack {
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.created.title",
                            defaultValue: "Your Safe Box has been created",
                            bundle: #bundle,
                            comment: "Success headline shown after a Safe Box is created."
                        )
                    )
                    .font(.system(.largeTitle))
                    Spacer()
                }
                .padding(.bottom, 10)

                HStack {
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.created.description",
                            defaultValue:
                                "Before you start using your Safe Box please safely store the configuration.",
                            bundle: #bundle,
                            comment:
                                "Instructional text on the Safe Box creationsuccess screen telling users to save configuration first."
                        )
                    )
                    .font(.system(.title3))
                    Spacer()
                }
                .padding(.bottom, 10)

                Divider()
                    .padding(.bottom, 10)

                RepoConfigInfo(
                    config: config,
                    onSave: {
                        saved = true
                    })
            }
            .padding()
        }
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button {
                    onContinue()
                } label: {
                    Text(
                        LocalizedStringResource(
                            "ios.repo_create.created.continue.button",
                            defaultValue: "Continue",
                            bundle: #bundle,
                            comment:
                                "Toolbar button on the Safe Box creation success screen to continue to the Safe Box files."
                        )
                    )
                }
                .disabled(!saved)
            }
        }
    }
}
