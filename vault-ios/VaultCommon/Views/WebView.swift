import SwiftUI
import WebKit

public enum WebViewAsset {
    case data(data: Data, mimeType: String, characterEncodingName: String)
    case url(url: URL)
}

public struct WebView: UIViewRepresentable {
    private let asset: WebViewAsset

    public init(asset: WebViewAsset) {
        self.asset = asset
    }

    public func makeUIView(context: Context) -> WKWebView {
        return WKWebView()
    }

    public func updateUIView(_ webView: WKWebView, context: Context) {
        switch asset {
        case .data(let data, let mimeType, let characterEncodingName):
            webView.load(
                data, mimeType: mimeType, characterEncodingName: characterEncodingName,
                baseURL: URL(string: "https://localhost")!)
        case .url(let url):
            webView.loadFileURL(url, allowingReadAccessTo: url)
        }
    }
}
