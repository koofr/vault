import Foundation
import XCTest

@testable import VaultUtils

final class StringUtilsTests: XCTestCase {
    func testReplaceLiteralTokenInsertsMarkdownLikeValueLiterally() throws {
        var attributedString = try AttributedString(markdown: "Value: TOKEN")

        attributedString.replaceLiteralToken("TOKEN", with: "[foo](bar) *** <br>")

        XCTAssertEqual(String(attributedString.characters), "Value: [foo](bar) *** <br>")
    }

    func testReplaceLiteralTokenReplacesAllOccurrences() throws {
        var attributedString = try AttributedString(markdown: "A TOKEN, B TOKEN, C TOKEN")

        attributedString.replaceLiteralToken("TOKEN", with: "x")

        XCTAssertEqual(String(attributedString.characters), "A x, B x, C x")
    }

    func testReplaceLiteralTokenLeavesStringUnchangedWhenTokenIsMissing() throws {
        let original = try AttributedString(markdown: "No token here")
        var attributedString = original

        attributedString.replaceLiteralToken("TOKEN", with: "x")

        XCTAssertEqual(attributedString, original)
    }

    func testReplaceLiteralTokenPreservesTokenPositionAttributes() throws {
        var attributedString = try AttributedString(markdown: "Do you want to delete **TOKEN**?")

        attributedString.replaceLiteralToken("TOKEN", with: "My file")

        let expected = try AttributedString(markdown: "Do you want to delete **My file**?")
        XCTAssertEqual(attributedString, expected)
    }

    func testReplaceLiteralTokenPreservesTokenAttributesWithUnderscoresInReplacement() throws {
        var attributedString = try AttributedString(markdown: "Do you want to delete **TOKEN**?")

        attributedString.replaceLiteralToken("TOKEN", with: "_My file_")

        let expected = try AttributedString(markdown: "Do you want to delete **\\_My file\\_**?")
        XCTAssertEqual(attributedString, expected)
    }

    func testReplaceLiteralTokenSupportsThematicBreakLikeInputLiterally() throws {
        var attributedString = try AttributedString(markdown: "Divider: TOKEN")

        attributedString.replaceLiteralToken("TOKEN", with: "---")

        XCTAssertEqual(String(attributedString.characters), "Divider: ---")
    }

    func testReplaceLiteralTokenSupportsMultipleDifferentTokens() throws {
        var attributedString = try AttributedString(
            markdown: "Delete **NAME_TOKEN** from _LOCATION_TOKEN_?")

        attributedString.replaceLiteralToken("NAME_TOKEN", with: "[foo](bar)")
        attributedString.replaceLiteralToken("LOCATION_TOKEN", with: "*** --- <br>")

        XCTAssertEqual(String(attributedString.characters), "Delete [foo](bar) from *** --- <br>?")
    }
}
