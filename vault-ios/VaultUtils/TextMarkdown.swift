import Foundation
import SwiftUI

extension AttributedString {
    /// Builds a localized `AttributedString` and applies a post-localization patch.
    ///
    /// Use this when translations include markdown styling plus literal placeholder
    /// tokens that should be replaced at runtime.
    ///
    /// - Important: Use explicit locale in `LocalizedStringResource` at call sites when
    ///   locale-driven rendering is required.
    ///
    /// - Parameters:
    ///   - resource: Localized resource whose markdown has already been parsed by Swift.
    ///   - patch: Mutation callback used to apply token replacement or other adjustments.
    /// - Returns: Patched attributed string ready for rendering in `Text`.
    public static func markdownLocalized(
        _ resource: LocalizedStringResource,
        patch: (inout AttributedString) -> Void = { _ in }
    ) -> AttributedString {
        markdown(AttributedString(localized: resource), patch: patch)
    }

    /// Applies a patch to an already prepared attributed string.
    ///
    /// This overload is useful in tests and in flows that construct attributed markdown
    /// outside of localization APIs.
    ///
    /// - Parameters:
    ///   - attributedString: Source attributed string to mutate.
    ///   - patch: Mutation callback used to replace tokens or tweak attributes.
    /// - Returns: Patched attributed string.
    public static func markdown(
        _ attributedString: AttributedString,
        patch: (inout AttributedString) -> Void = { _ in }
    ) -> AttributedString {
        var mutableAttributedString = attributedString
        patch(&mutableAttributedString)
        return mutableAttributedString
    }
}

extension Text {
    /// Convenience wrapper around `AttributedString.markdownLocalized(_:patch:)`
    /// that returns `Text`.
    ///
    /// - Parameters:
    ///   - resource: Localized resource whose markdown has already been parsed by Swift.
    ///   - patch: Mutation callback used to apply token replacement or other adjustments.
    /// - Returns: `Text` initialized from the patched attributed string.
    public static func markdownLocalized(
        _ resource: LocalizedStringResource,
        patch: (inout AttributedString) -> Void = { _ in }
    ) -> Text {
        Text(AttributedString.markdownLocalized(resource, patch: patch))
    }

    /// Convenience wrapper around `AttributedString.markdown(_:patch:)` that returns
    /// `Text`.
    ///
    /// - Parameters:
    ///   - attributedString: Source attributed string to mutate.
    ///   - patch: Mutation callback used to replace tokens or tweak attributes.
    /// - Returns: `Text` initialized from the patched attributed string.
    public static func markdown(
        _ attributedString: AttributedString,
        patch: (inout AttributedString) -> Void = { _ in }
    ) -> Text {
        Text(AttributedString.markdown(attributedString, patch: patch))
    }
}
