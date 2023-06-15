import SwiftUI
import VaultMobile

public struct NotificationView: View {
    public var message: String

    public var body: some View {
        HStack {
            HStack {
                VStack(alignment: .leading, spacing: 3) {
                    Text(message).font(.system(.body)).foregroundColor(
                        Color(UIColor(rgb: 0xE8E9ED)))
                }

                Spacer()

                Text("Dismiss").font(.system(.body)).bold().foregroundColor(
                    Color(UIColor(rgb: 0x90B2F1)))
            }
            .padding(.vertical, 15)
            .padding(.horizontal, 17)
            .background(Color(UIColor(rgb: 0x202125)))
            .cornerRadius(5)
        }
        .padding(.horizontal, 10)
        .padding(.top, 10)
        .transition(.move(edge: .bottom))
        .animation(.easeInOut, value: true)
    }
}

public struct NotificationView_Previews: PreviewProvider {
    static public var previews: some View {
        VStack(spacing: 0) {
            Spacer()

            NotificationView(message: "Hello")

            NotificationView(
                message:
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non ante aliquam, suscipit lacus ut, tincidunt ex. Aenean bibendum id nibh sed rutrum. Nulla vitae ante sit amet sapien porta mattis at sit amet metus."
            )
        }
    }
}
