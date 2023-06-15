import Foundation

public enum ImageData {
    case jpeg(data: Data)
    case png(data: Data)

    public var data: Data {
        switch self {
        case .jpeg(let data): return data
        case .png(let data): return data
        }
    }

    public var ext: String {
        switch self {
        case .jpeg(_): return "jpg"
        case .png(_): return "png"
        }
    }
}

public func getImageData(_ image: UIImage) -> ImageData? {
    let pngData = image.pngData()

    if pngData != nil && isImageTransparent(image) {
        return .png(data: pngData!)
    }

    let jpegData = image.jpegData(compressionQuality: 1.0)

    if jpegData == nil {
        if let data = pngData {
            return .png(data: data)
        } else {
            return nil
        }
    }

    if pngData == nil {
        if let data = jpegData {
            return .jpeg(data: data)
        } else {
            return nil
        }
    }

    if jpegData!.count < pngData!.count {
        return .jpeg(data: jpegData!)
    }

    return .png(data: pngData!)
}

public func isImageTransparent(_ image: UIImage) -> Bool {
    guard let alpha: CGImageAlphaInfo = image.cgImage?.alphaInfo else { return false }

    return alpha == .first || alpha == .last || alpha == .premultipliedFirst
        || alpha == .premultipliedLast
}
