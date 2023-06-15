import SwiftUI
import VaultMobile

public struct RepoUnlockForm: View {
    public var info: RepoUnlockInfo
    public var onUnlock: (String) -> Void
    public var message: String

    @State var password: String = ""

    var isLoading: Bool {
        switch info.status {
        case .loading(_): return true
        default: return false
        }
    }

    init(info: RepoUnlockInfo, onUnlock: @escaping (String) -> Void, message: String? = nil) {
        self.info = info
        self.onUnlock = onUnlock
        self.message = message ?? "Enter your Safe Key to continue"
    }

    public var body: some View {
        VStack {
            Spacer()

            Text(info.repoName ?? "")
                .font(.largeTitle)
                .fontWeight(.semibold)
                .padding(.bottom, 15)

            Text(message)
                .font(.headline)
                .multilineTextAlignment(.center)
                .padding(.bottom, 20)

            switch info.status {
            case .err(let error, _):
                Text(error)
                    .font(.body)
                    .foregroundColor(Color(.systemRed))
                    .multilineTextAlignment(.center)
                    .padding(.bottom, 20)
            default:
                EmptyView()
            }

            RepoPasswordField(
                text: $password,
                onSubmit: {
                    onUnlock(password)
                }, autoFocus: true
            )
            .padding(.bottom, 20)

            Button {
                onUnlock(password)
            } label: {
                VStack {
                    Spacer()
                    HStack {
                        Spacer()
                        Text("Continue").font(.system(size: 20, weight: .bold))
                            .foregroundColor(.white)
                        Spacer()
                    }
                    Spacer()
                }
            }
            .buttonStyle(UnlockButtonStyle())
            .frame(width: 300, height: 60)
            .disabled(isLoading)

            Spacer()
        }
        .frame(width: 300)
        .padding()
        .overlay {
            if isLoading {
                ProgressView("Unlocking…")
            }
        }
    }
}

struct UnlockButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled: Bool

    func makeBody(configuration: Self.Configuration) -> some View {
        configuration.label
            .background(
                isEnabled
                    ? (configuration.isPressed
                        ? Color(UIColor(rgb: 0x67a200))
                        : Color(UIColor(rgb: 0x71ba05)))
                    : Color(UIColor(rgb: 0xa8adaf))
            )
            .cornerRadius(3)
    }

}

public struct RepoUnlockForm_Previews: PreviewProvider {
    static public var previews: some View {
        RepoUnlockForm(info: RepoUnlockInfo(status: .loaded, repoName: "My safe box")) { password in
        }
    }
}
