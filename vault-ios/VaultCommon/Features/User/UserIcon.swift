import SwiftUI
import VaultMobile

public struct UserIcon: View {
    public let container: Container

    @ObservedObject private var userProfilePictureLoaded: Subscription<Bool>
    @ObservedObject private var user: Subscription<User>

    @State private var profilePicture: UIImage?

    public init(container: Container) {
        self.container = container

        userProfilePictureLoaded = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.userProfilePictureLoadedSubscribe(cb: cb)
            },
            getData: { v, id in
                v.userProfilePictureLoadedData(id: id)
            })

        user = Subscription(
            mobileVault: container.mobileVault,
            subscribe: { v, cb in
                v.userSubscribe(cb: cb)
            },
            getData: { v, id in
                v.userData(id: id)
            })

        container.mobileVault.userEnsureProfilePicture()
    }

    private func loadProfilePicture() {
        guard let bytes = container.mobileVault.userGetProfilePicture() else {
            return
        }

        profilePicture = UIImage(data: Data(bytes))
    }

    public var body: some View {
        VStack {
            if !userProfilePictureLoaded.data! {
                Spacer()
                    .frame(width: 40, height: 40)
            } else if let profilePicture = profilePicture {
                Image(uiImage: profilePicture)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .clipShape(Circle())
                    .frame(width: 40, height: 40)
            } else {
                UserIconFallback(name: user.data?.fullName)
            }
        }
        .onChange(
            of: userProfilePictureLoaded.data!,
            perform: { loaded in
                loadProfilePicture()
            }
        )
        .onAppear {
            loadProfilePicture()
        }
    }
}

public struct UserIconFallback: View {
    var name: String?

    public init(name: String?) {
        self.name = name
    }

    public var body: some View {
        VStack(alignment: .center) {
            Spacer()
            HStack(alignment: .center) {
                Spacer()
                Text(name?.prefix(1).uppercased() ?? "")
                    .bold()
                    .foregroundColor(.white)
                Spacer()
            }
            Spacer()
        }
        .background(Color(UIColor(rgb: 0xd4d6d7)))
        .clipShape(Circle())
        .frame(width: 40, height: 40)
    }
}
