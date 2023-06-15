import Foundation
import VaultMobile

public enum UploadFileData {
    case data(data: Data)
    case file(path: String, removeFileAfterUpload: Bool)
}

public struct UploadFile {
    let name: String
    let size: Int64?
    let data: UploadFileData
}
