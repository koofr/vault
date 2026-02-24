import Foundation
import SwiftUI
import XCTest

@testable import VaultUtils

final class TextMarkdownTests: XCTestCase {
    func testAttributedStringMarkdownPreservesParagraphNewlines() {
        let input = AttributedString("First paragraph\n\nSecond paragraph")

        let attributedString = AttributedString.markdown(input)

        XCTAssertEqual(String(attributedString.characters), "First paragraph\n\nSecond paragraph")
    }

    func testAttributedStringMarkdownAppliesPatchAfterLocalization() {
        let token = "TOKEN"
        let input = AttributedString("Delete \(token)?")

        let attributedString = AttributedString.markdown(input) { attributedString in
            attributedString.replaceLiteralToken(token, with: "[foo](bar)")
        }

        XCTAssertEqual(String(attributedString.characters), "Delete [foo](bar)?")
    }
}
