import SwiftUI

public struct EmptyFolderView: View {
    public var body: some View {
        HStack {
            Spacer()

            VStack {
                Image(systemName: "folder.fill")
                    .font(.system(size: 50))
                    .foregroundColor(Color(.systemGray))
                    .padding(.bottom, 10)

                Text("Folder is Empty")
                    .font(.system(size: 18, weight: .bold))
            }

            Spacer()
        }
    }
}
