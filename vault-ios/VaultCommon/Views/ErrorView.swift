import SwiftUI

public struct ErrorView: View {
    public let errorText: String
    public let onRetry: (() -> Void)?

    public init(errorText: String, onRetry: (() -> Void)? = nil) {
        self.errorText = errorText
        self.onRetry = onRetry
    }

    public var body: some View {
        VStack {
            Text(
                LocalizedStringResource(
                    "ios.views.error.title",
                    defaultValue: "Error",
                    bundle: #bundle,
                    comment: "Title text shown at the top of the generic error view."
                )
            )
            .font(.title)
            .padding(.bottom, 20)

            Text(errorText).multilineTextAlignment(.center).padding(.bottom, 20)

            if let onRetry = onRetry {
                Button {
                    onRetry()
                } label: {
                    Text(
                        LocalizedStringResource(
                            "ios.views.error.try_again.button",
                            defaultValue: "Try again",
                            bundle: #bundle,
                            comment:
                                "Button label in the generic error view to retry the failed action."
                        )
                    )
                    .foregroundColor(Color(.link))
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding()
    }
}
