import SwiftUI

public struct LoadingView: View {
    public var body: some View {
        ProgressView {
            Text(
                LocalizedStringResource(
                    "ios.views.loading.label",
                    defaultValue: "Loading…",
                    bundle: #bundle,
                    comment: "Caption shown below the spinner in the shared loading view."
                )
            )
        }
    }
}
