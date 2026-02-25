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

                Text(
                    LocalizedStringResource(
                        "ios.views.empty_folder.title",
                        defaultValue: "Folder is Empty",
                        bundle: #bundle,
                        comment: "Empty-state title shown when a folder has no files."
                    )
                )
                .font(.system(size: 18, weight: .bold))
            }

            Spacer()
        }
    }
}
