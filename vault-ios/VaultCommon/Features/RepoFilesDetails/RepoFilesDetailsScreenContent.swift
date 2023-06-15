import AVKit
import Foundation
import MediaPlayer
import PhotosUI
import VaultMobile

public enum RepoFilesDetailsScreenContentData {
    case image(image: UIImage)
    case gifImage(image: UIImage)
    case media(player: AVPlayer)
    case webViewAsset(asset: WebViewAsset)
    case error(error: String)

    public static let imageExts: Set = [
        "bmp", "cur", "gif", "heic", "ico", "jpeg", "jpg", "png", "tiff", "webp", "xbm",
    ]
    public static let mediaContentTypes: Set = Set(AVURLAsset.audiovisualMIMETypes())
    public static let webViewAssetURLExts: Set = [
        "doc", "docm", "docx", "pdf", "pot", "potm", "potx", "pps", "ppsm", "ppsx", "ppt", "pptm",
        "pptx", "rtf", "xls", "xlsm", "xlsx",
    ]

    public static func getLoader(file: RepoFile, onWarning: @escaping (String) -> Void) -> (
        (URL) async -> RepoFilesDetailsScreenContentData
    )? {
        if let ext = file.ext {
            if RepoFilesDetailsScreenContentData.imageExts.contains(ext) {
                return { localFileURL in
                    do {
                        let data = try Data(contentsOf: localFileURL)

                        if data.count >= 3 && data[0...2].elementsEqual("GIF".utf8) {
                            if let image = VaultCommon.gifImage(data: data) {
                                return .gifImage(image: image)
                            } else {
                                return .error(error: "Invalid image")
                            }
                        } else {
                            if let image = UIImage(data: data) {
                                return .image(image: image)
                            } else {
                                return .error(error: "Invalid image")
                            }
                        }
                    } catch {
                        return .error(error: error.localizedDescription)
                    }
                }
            } else if RepoFilesDetailsScreenContentData.webViewAssetURLExts.contains(ext) {
                return { localFileURL in
                    return .webViewAsset(asset: .url(url: localFileURL))
                }
            }
        }

        if let contentType = file.contentType {
            if RepoFilesDetailsScreenContentData.mediaContentTypes.contains(contentType) {
                return { localFileURL in
                    let asset = AVAsset(url: localFileURL)

                    let playerItem = AVPlayerItem(asset: asset)

                    do {
                        var metadata = try await asset.load(.metadata)

                        if metadata.first(where: { $0.commonKey == AVMetadataKey.commonKeyTitle })
                            == nil
                        {
                            let titleItem = AVMutableMetadataItem()
                            titleItem.identifier = AVMetadataIdentifier.commonIdentifierTitle
                            titleItem.value = file.name as (NSCopying & NSObjectProtocol)?
                            metadata.append(titleItem)
                        }

                        playerItem.externalMetadata = metadata
                    } catch {
                        onWarning("Failed to load asset metadata \(error)")
                    }

                    let player = AVPlayer(playerItem: playerItem)

                    return .media(player: player)
                }
            }
        }

        if file.category == .text || file.category == .code {
            return { localFileURL in
                do {
                    let data = try Data(contentsOf: localFileURL)

                    return .webViewAsset(
                        asset: .data(
                            data: data, mimeType: "text/plain", characterEncodingName: "utf-8"))
                } catch {
                    return .error(error: error.localizedDescription)
                }
            }
        }

        return nil
    }
}

public enum RepoFilesDetailsScreenContent {
    case loading
    case downloading
    case downloaded(localFileURL: URL, data: RepoFilesDetailsScreenContentData)
    case notSupported(file: RepoFile)
}
