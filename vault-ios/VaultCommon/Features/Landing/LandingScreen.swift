import SwiftUI

public struct LandingScreen: View {
    public let container: Container

    @Environment(\.colorScheme) var colorScheme

    @State private var isSigningIn: Bool = false

    public var body: some View {
        NavigationView {
            VStack {
                Image(colorScheme == .dark ? "landing-logo-dark" : "landing-logo")
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: 233)
                    .padding(.bottom, 50)

                Text("One vault for all\nyour private files.")
                    .font(.system(size: 32, weight: .bold))
                    .multilineTextAlignment(.center)
                    .padding(.bottom, 20)

                Text(
                    "Powerful, open source client-side encryption. Unlock enhanced security for your most sensitive files."
                )
                .font(.system(size: 18))
                .multilineTextAlignment(.center)
                .frame(width: 313)
                .padding(.bottom, 32)

                Image(colorScheme == .dark ? "landing-graphic-dark" : "landing-graphic")
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: 313)
                    .padding(.bottom, 30)

                Button {
                    isSigningIn = true

                    container.authHelper.login {
                        isSigningIn = false
                    }
                } label: {
                    VStack {
                        Spacer()
                        HStack {
                            Spacer()
                            Text("Get started").font(.system(size: 20, weight: .bold))
                                .foregroundColor(.white)
                            Spacer()
                        }
                        Spacer()
                    }
                }
                .buttonStyle(LandingButtonStyle())
                .frame(width: 300, height: 60)
                .disabled(isSigningIn)
            }
            .padding()
            .onAppear {
                isSigningIn = false
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        container.sheets.show(name: "infoSheet") { _, hide in
                            InfoSheet(container: container, onDismiss: hide)
                        }
                    } label: {
                        Image(systemName: "info.circle")
                    }
                }
            }
        }
        .navigationViewStyle(.stack)
    }
}

struct LandingButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled: Bool

    func makeBody(configuration: Self.Configuration) -> some View {
        configuration.label
            .background(
                isEnabled
                    ? (configuration.isPressed
                        ? Color(UIColor(rgb: 0x0576f1))
                        : Color(UIColor(rgb: 0x1683fb)))
                    : Color(UIColor(rgb: 0xa8adaf))
            )
            .cornerRadius(3)
    }

}
