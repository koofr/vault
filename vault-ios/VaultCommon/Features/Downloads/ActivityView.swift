import SwiftUI

public struct ActivityView: UIViewControllerRepresentable {
    public let activityItems: [Any]
    public let showOpenInDownloads: Bool

    init(activityItems: [Any], showOpenInDownloads: Bool) {
        self.activityItems = activityItems
        self.showOpenInDownloads = showOpenInDownloads
    }

    public func makeUIViewController(context: UIViewControllerRepresentableContext<ActivityView>)
        -> UIActivityViewController
    {
        var applicationActivities = [UIActivity]()

        if showOpenInDownloads {
            applicationActivities.append(OpenDownloadsFolderActivity())
        }

        return UIActivityViewController(
            activityItems: activityItems, applicationActivities: applicationActivities)
    }

    public func updateUIViewController(
        _ uiViewController: UIActivityViewController,
        context: UIViewControllerRepresentableContext<ActivityView>
    ) {}
}
