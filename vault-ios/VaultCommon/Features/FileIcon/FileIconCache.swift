import UIKit
import VaultMobile

public class FileIconCache {
    private let mobileVault: MobileVault

    private var cache: [FileIconCacheKey: UIImage] = [:]

    public init(mobileVault: MobileVault) {
        self.mobileVault = mobileVault
    }

    public func getIcon(props: FileIconProps, scale: Int) -> UIImage {
        let key = FileIconCacheKey(props: props, scale: scale)

        if let icon = cache[key] {
            return icon
        } else {
            let icon = buildIcon(props: props, scale: scale)
            cache[key] = icon
            return icon
        }
    }

    private func buildIcon(props: FileIconProps, scale: Int) -> UIImage {
        let png = mobileVault.fileIconPng(props: props, scale: UInt32(scale))

        return UIImage(data: Data(png.png), scale: Double(scale))!
    }
}

struct FileIconCacheKey: Hashable {
    let props: FileIconProps
    let scale: Int
}
