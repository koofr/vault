import Foundation

extension AttributedString {
    /// Replaces every occurrence of a token with literal runtime text.
    ///
    /// This is intended for markdown-backed localized strings that contain placeholders
    /// (for example `**__NAME__**`) where the replacement value must not be interpreted
    /// as markdown.
    ///
    /// Behavior:
    /// - Finds all exact occurrences of `token`.
    /// - Builds the replacement from `AttributedString(value)`, which treats `value` as
    ///   plain text.
    /// - Merges attributes from the token range onto the replacement so surrounding inline
    ///   styling at the placeholder location is preserved.
    ///
    /// - Parameters:
    ///   - token: Literal placeholder token to replace.
    ///   - value: Literal text inserted for each token occurrence.
    public mutating func replaceLiteralToken(_ token: String, with value: String) {
        while let range = self.range(of: token) {
            let attrs = self[range].runs.first?.attributes ?? AttributeContainer()
            var replacement = AttributedString(value)  // literal text, not markdown
            replacement.mergeAttributes(attrs)  // keep style at token position
            self.replaceSubrange(range, with: replacement)
        }
    }
}
