import SwiftUI

public struct ErrorView: View {
    public let errorText: String
    public let onRetry: (() -> Void)?

    public init(errorText: String, onRetry: (() -> Void)?) {
        self.errorText = errorText
        self.onRetry = onRetry
    }

    public var body: some View {
        VStack {
            Text("Error").font(.title).padding(.bottom, 20)

            Text(errorText).multilineTextAlignment(.center).padding(.bottom, 20)

            if let onRetry = onRetry {
                Button {
                    onRetry()
                } label: {
                    Text("Try again")
                        .foregroundColor(Color(.link))
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding()
    }
}
