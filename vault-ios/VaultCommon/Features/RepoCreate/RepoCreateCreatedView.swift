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
                    Text("Your Safe Box has been created.")
                        .font(.system(.largeTitle))
                    Spacer()
                }
                .padding(.bottom, 10)

                HStack {
                    Text(
                        "Before you start using your Safe Box please safely store the configuration."
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
        .navigationTitle(Text("Create a new Safe Box"))
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Continue") {
                    onContinue()
                }
                .disabled(!saved)
            }
        }
    }
}
