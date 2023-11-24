import UIKit

extension UIActivity.ActivityType {
    public static let openDownloadsFolderActivityType = UIActivity.ActivityType(
        rawValue: "net.koofr.vault.OpenDownloadsFolderActivity")
}

public class OpenDownloadsFolderActivity: UIActivity {
    private var activityItems: [Any]?

    public override class var activityCategory: UIActivity.Category {
        return .action
    }

    public override var activityType: ActivityType? {
        return .openDownloadsFolderActivityType
    }

    public override var activityTitle: String? {
        return "Open in downloads"
    }

    public override var activityImage: UIImage? {
        return UIImage(systemName: "arrow.down.to.line.circle")
    }

    public override func canPerform(withActivityItems activityItems: [Any]) -> Bool {
        return true
    }

    public override func prepare(withActivityItems activityItems: [Any]) {
        self.activityItems = activityItems
    }

    public override func perform() {
        if let activityItems = activityItems {
            if let activityItem = activityItems.first {
                if let url = activityItem as? URL {
                    let parentURL = url.deletingLastPathComponent()

                    if var components = URLComponents(
                        url: parentURL, resolvingAgainstBaseURL: false)
                    {
                        components.scheme = "shareddocuments"

                        if let url = components.url {
                            UIApplication.shared.open(url)
                        }
                    }
                }
            }
        }

        activityDidFinish(true)
    }
}
