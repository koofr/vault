import Foundation
import UIKit
import UniformTypeIdentifiers
import VaultMobile

// used for PasteButton supportedContentTypes
public let uploadHelperUTTypes = [
    UTType.plainText,
    UTType.utf8PlainText,
    UTType.text,
    UTType.html,
    UTType.rtf,
    UTType.image,
    UTType.vCard,
    UTType.fileURL,
    UTType.folder,
    UTType.directory,
    UTType.jpeg,
    UTType.png,
    UTType.gif,
    UTType.tiff,
    // url is not included in uploadHelperUTTypes because we want the cliboard
    // data to be in text format not binary plist
    // UTType.url,
    UTType.webP,
    UTType.mpeg4Movie,
    UTType.quickTimeMovie,
]

public class UploadHelper {
    private let mobileVault: MobileVault
    private let storageHelper: StorageHelper

    public init(mobileVault: MobileVault, storageHelper: StorageHelper) {
        self.mobileVault = mobileVault
        self.storageHelper = storageHelper
    }

    public func uploadFiles(repoId: String, encryptedParentPath: String, files: [UploadFile]) {
        for file in files {
            switch file.data {
            case .data(let data):
                mobileVault.transfersUploadBytes(
                    repoId: repoId, encryptedParentPath: encryptedParentPath, name: file.name,
                    bytes: data)
            case .file(let path, let removeFileAfterUpload):
                mobileVault.transfersUploadFile(
                    repoId: repoId, encryptedParentPath: encryptedParentPath, name: file.name,
                    localFilePath: path,
                    removeFileAfterUpload: removeFileAfterUpload)
            }
        }
    }

    public func uploadSecurityScopedResources(
        repoId: String, encryptedParentPath: String, urls: [URL]
    )
        throws
    {
        var uploadFiles = [UploadFile]()

        for url in urls {
            try securityScopedResourceToUploadFiles(url: url, uploadFiles: &uploadFiles)
        }

        self.uploadFiles(
            repoId: repoId, encryptedParentPath: encryptedParentPath, files: uploadFiles)
    }

    private func securityScopedResourceToUploadFiles(url: URL, uploadFiles: inout [UploadFile])
        throws
    {
        if !url.startAccessingSecurityScopedResource() {
            return
        }

        defer {
            url.stopAccessingSecurityScopedResource()
        }

        // create a copy of the file/folder. right now it is too difficult to track when to call stopAccessingSecurityScopedResource() if we are uploading folders (it should be called at the end of the last uploaded file in the folder)
        let tempFileURL = self.storageHelper.getTempDir().appendingPathComponent(UUID().uuidString)

        try FileManager.default.copyItem(at: url, to: tempFileURL)

        for file in self.urlToUploadFiles(
            url: tempFileURL, suggestedName: url.lastPathComponent, removeFileAfterUpload: true)
        {
            uploadFiles.append(file)
        }
    }

    @MainActor
    public func itemProvidersToFiles(itemProviders: [NSItemProvider], loadFileRepresentation: Bool)
        async -> [UploadFile]
    {
        var files = [UploadFile]()

        for itemProvider in itemProviders {
            for file in await itemProviderToFiles(
                itemProvider: itemProvider, loadFileRepresentation: loadFileRepresentation)
            {
                files.append(file)
            }
        }

        return files
    }

    @MainActor
    public func itemProviderToFiles(itemProvider: NSItemProvider, loadFileRepresentation: Bool)
        async -> [UploadFile]
    {
        print("UploadHelper content types \(itemProvider.registeredContentTypes)")

        let suggestedName = itemProvider.suggestedName

        for contentType in itemProvider.registeredContentTypes {
            do {
                let files = try await itemProviderContentTypeToFiles(
                    itemProvider: itemProvider, contentType: contentType,
                    suggestedName: suggestedName, loadFileRepresentation: loadFileRepresentation)

                if !files.isEmpty {
                    return files
                }
            } catch {
                mobileVault.notificationsShow(
                    message: "Failed to handle content type: \(contentType.identifier): \(error)")
            }
        }

        let contentTypesString = itemProvider.registeredContentTypes.map { $0.identifier }.joined(
            separator: ", ")

        mobileVault.notificationsShow(message: "Unknown content types: \(contentTypesString)")

        return []
    }

    @MainActor
    private func itemProviderContentTypeToFiles(
        itemProvider: NSItemProvider, contentType: UTType, suggestedName: String?,
        loadFileRepresentation: Bool
    ) async throws -> [UploadFile] {
        if let handler = itemProviderContentTypeHandler(
            itemProvider: itemProvider, contentType: contentType)
        {
            let coding = try await loadItemProviderData(
                itemProvider: itemProvider, contentType: contentType,
                loadFileRepresentation: loadFileRepresentation)

            let removeFileAfterUpload = loadFileRepresentation

            return try await handler(contentType, coding, suggestedName, removeFileAfterUpload)
        } else {
            return []
        }
    }

    @MainActor
    private func loadItemProviderData(
        itemProvider: NSItemProvider, contentType: UTType, loadFileRepresentation: Bool
    ) async throws -> NSSecureCoding {
        if loadFileRepresentation {
            return try await loadItemProviderFileData(
                itemProvider: itemProvider, contentType: contentType)
        } else {
            return try await itemProvider.loadItem(forTypeIdentifier: contentType.identifier)
        }
    }

    @MainActor
    private func loadItemProviderFileData(itemProvider: NSItemProvider, contentType: UTType)
        async throws -> NSSecureCoding
    {
        return try await withCheckedThrowingContinuation { continuation in
            let _ = itemProvider.loadFileRepresentation(for: contentType) { url, _, err in
                if let url = url {
                    do {
                        let tempFileURL = self.storageHelper.getTempDir().appendingPathComponent(
                            UUID().uuidString)

                        try FileManager.default.copyItem(at: url, to: tempFileURL)

                        continuation.resume(with: .success(tempFileURL as NSSecureCoding))
                    } catch {
                        continuation.resume(with: .failure(error))
                    }
                } else if let err = err {
                    continuation.resume(with: .failure(err))
                } else {
                    enum MissingUrlError: Error {
                        case missingUrl
                    }

                    continuation.resume(with: .failure(MissingUrlError.missingUrl))
                }
            }
        }
    }

    private func itemProviderContentTypeHandler(itemProvider: NSItemProvider, contentType: UTType)
        -> ((UTType, NSSecureCoding, String?, Bool) async throws -> [UploadFile])?
    {
        // keep this in sync with uploadHelperUTTypes
        switch contentType {
        case UTType.plainText:
            return plainTextHandler
        case UTType.utf8PlainText:
            return plainTextHandler
        case UTType.text:
            return textHandler
        case UTType.url:
            return urlHandler
        case UTType.rtf:
            return rtfHandler
        case UTType.html:
            return htmlHandler
        case UTType.image:
            return imageHandler
        case UTType.vCard:
            return vCardHandler
        case UTType.fileURL:
            return fileHandler(nil)
        case UTType.folder:
            return fileHandler("zip")
        case UTType.directory:
            return fileHandler("zip")
        case UTType.jpeg:
            return fileHandler("jpg")
        case UTType.png:
            return fileHandler("png")
        case UTType.gif:
            return fileHandler("gif")
        case UTType.tiff:
            return fileHandler("tiff")
        case UTType.webP:
            return fileHandler("webp")
        case UTType.mpeg4Movie:
            return fileHandler("mp4")
        case UTType.quickTimeMovie:
            return fileHandler("mov")
        default:
            return nil
        }
    }

    private func plainTextHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let data = coding as? Data {
            let name = getRandomName(base: "text", suffix: "txt")

            return [uploadFileData(name: name, data: data)]
        } else if let content = coding as? String {
            let name = getRandomName(base: "text", suffix: "txt")

            return [uploadFileString(name: name, content: content)]
        } else {
            return []
        }
    }

    private func textHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let data = coding as? Data {
            let name = getRandomName(base: "text", suffix: "txt")

            return [uploadFileData(name: name, data: data)]
        } else if let content = coding as? String {
            let name = getRandomName(base: "text", suffix: "txt")

            return [uploadFileString(name: name, content: content)]
        } else {
            return []
        }
    }

    private func urlHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let content = coding as? String {
            let name = getRandomName(base: "link", suffix: "txt")

            return [uploadFileString(name: name, content: content)]
        } else if let url = coding as? NSURL {
            let name = getRandomName(base: "link", suffix: "txt")

            if let urlString = url.absoluteString {
                return [uploadFileString(name: name, content: urlString)]
            } else {
                return []
            }
        } else {
            return []
        }
    }

    private func htmlHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let data = coding as? Data {
            let name = getRandomName(base: "text", suffix: "txt")

            let attributedString = try NSAttributedString(
                data: data, options: [.documentType: NSAttributedString.DocumentType.html],
                documentAttributes: nil)

            return [uploadFileString(name: name, content: attributedString.string)]
        } else {
            return []
        }
    }

    private func rtfHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let data = coding as? Data {
            let name = getRandomName(base: "text", suffix: "txt")

            let attributedString = try NSAttributedString(data: data, documentAttributes: nil)

            return [uploadFileString(name: name, content: attributedString.string)]
        } else {
            return []
        }
    }

    private func imageHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let image = coding as? UIImage {
            if let imageData = getImageData(image) {
                let name = getRandomName(base: "image", suffix: imageData.ext)

                return [uploadFileData(name: name, data: imageData.data)]
            } else {
                return []
            }
        } else {
            return []
        }
    }

    private func vCardHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) throws -> [UploadFile] {
        if let data = coding as? Data {
            let name = getRandomName(base: "contact", suffix: "vcf")

            return [uploadFileData(name: name, data: data)]
        } else {
            return []
        }
    }

    private func fileHandler(_ ext: String?) -> (UTType, NSSecureCoding, String?, Bool) async throws
        -> [UploadFile]
    {
        return { contentType, coding, suggestedName, removeFileAfterUpload in
            if let data = coding as? Data {
                return try await self.fileDataHandler(contentType, data, ext)
            }

            if let suggestedName = suggestedName {
                if let ext = ext {
                    return self.fileURLHandler(
                        contentType, coding, "\(suggestedName).\(ext)", removeFileAfterUpload)
                }
            }

            return self.fileURLHandler(contentType, coding, nil, removeFileAfterUpload)
        }
    }

    private func fileDataHandler(_ contentType: UTType, _ data: Data, _ ext: String?) async throws
        -> [UploadFile]
    {
        let name = getRandomName(base: "file", suffix: ext)

        if data.count > 4 * 1024 * 1024 {
            let tempFileURL = self.storageHelper.getTempDir().appendingPathComponent(
                UUID().uuidString)

            try data.write(to: tempFileURL)

            let uploadFile = UploadFile(
                name: name, size: Int64(data.count),
                data: .file(path: tempFileURL.path, removeFileAfterUpload: true))

            return [uploadFile]
        } else {
            return [uploadFileData(name: name, data: data)]
        }
    }

    private func fileURLHandler(
        _ contentType: UTType, _ coding: NSSecureCoding, _ suggestedName: String?,
        _ removeFileAfterUpload: Bool
    ) -> [UploadFile] {
        if let url = coding as? URL {
            return urlToUploadFiles(
                url: url, suggestedName: suggestedName, removeFileAfterUpload: removeFileAfterUpload
            )
        } else {
            return []
        }
    }

    public func urlToUploadFiles(url: URL, suggestedName: String?, removeFileAfterUpload: Bool)
        -> [UploadFile]
    {
        if !url.isFileURL {
            print("UploadHelper URL is not a file: \(url)")

            return []
        }

        var uploadFiles = [UploadFile]()

        urlToUploadFileHandleURL(
            url: url, namePrefix: nil, suggestedName: suggestedName,
            removeFileAfterUpload: removeFileAfterUpload, uploadFiles: &uploadFiles)

        return uploadFiles
    }

    private func urlToUploadFileHandleURL(
        url: URL, namePrefix: String?, suggestedName: String?, removeFileAfterUpload: Bool,
        uploadFiles: inout [UploadFile]
    ) {
        do {
            let resourceValues = try url.resourceValues(forKeys: [.isDirectoryKey])

            if let isDirectory = resourceValues.isDirectory {
                if isDirectory {
                    urlToUploadFileHandleDir(
                        url: url, namePrefix: namePrefix, suggestedName: suggestedName,
                        removeFileAfterUpload: removeFileAfterUpload, uploadFiles: &uploadFiles)
                } else {
                    urlToUploadFileHandleFile(
                        url: url, namePrefix: namePrefix, suggestedName: suggestedName,
                        removeFileAfterUpload: removeFileAfterUpload, uploadFiles: &uploadFiles)
                }
            }
        } catch {
            mobileVault.notificationsShow(message: "Failed to handle file: \(error)")
        }
    }

    private func urlToUploadFileHandleDir(
        url: URL, namePrefix: String?, suggestedName: String?, removeFileAfterUpload: Bool,
        uploadFiles: inout [UploadFile]
    ) {
        do {
            let name = suggestedName ?? url.lastPathComponent

            let dirNamePrefix = "\(namePrefix ?? "")\(name)/"

            var childURLs = try FileManager.default.contentsOfDirectory(
                at: url, includingPropertiesForKeys: nil)

            childURLs.sort {
                $0.absoluteString.localizedCompare($1.absoluteString) == .orderedAscending
            }

            for childURL in childURLs {
                urlToUploadFileHandleURL(
                    url: childURL, namePrefix: dirNamePrefix, suggestedName: nil,
                    removeFileAfterUpload: removeFileAfterUpload, uploadFiles: &uploadFiles)
            }
        } catch {
            mobileVault.notificationsShow(message: "Failed to handle folder: \(error)")
        }
    }

    private func urlToUploadFileHandleFile(
        url: URL, namePrefix: String?, suggestedName: String?, removeFileAfterUpload: Bool,
        uploadFiles: inout [UploadFile]
    ) {
        do {
            var name = suggestedName ?? url.lastPathComponent

            if let namePrefix = namePrefix {
                name = namePrefix + name
            }

            let path = url.path(percentEncoded: false)

            let attrs = try FileManager.default.attributesOfItem(atPath: path)

            let size = (attrs[.size] as? UInt64).map { Int64($0) }

            let uploadFile = UploadFile(
                name: name, size: size,
                data: .file(path: path, removeFileAfterUpload: removeFileAfterUpload))

            uploadFiles.append(uploadFile)
        } catch {
            mobileVault.notificationsShow(message: "Failed to handle file: \(error)")
        }
    }

    private func getRandomName(base: String, suffix: String?) -> String {
        var name = "\(base)-\(UUID().uuidString.prefix(8).lowercased())"

        if let suffix = suffix {
            name = "\(name).\(suffix)"
        }

        return name
    }

    private func uploadFileString(name: String, content: String) -> UploadFile {
        return uploadFileData(name: name, data: content.data(using: .utf8)!)
    }

    private func uploadFileData(name: String, data: Data) -> UploadFile {
        return UploadFile(name: name, size: Int64(data.count), data: .data(data: data))
    }
}
