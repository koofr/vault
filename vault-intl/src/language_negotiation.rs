// Copied from
// https://github.com/projectfluent/fluent-langneg-rs/blob/945d2d8cbd1544948963743c327563a3ba834924/src/negotiate/mod.rs

use icu_locale::{LocaleExpander, TransformResult};
use icu_locale_core::LanguageIdentifier;

use crate::icu_data_provider::IcuDataProvider;

lazy_static::lazy_static! {
    static ref LOCALE_EXPANDER: LocaleExpander = LocaleExpander::try_new_common_unstable(&IcuDataProvider)
        .expect("failed to build locale expander");
}

fn subtag_matches<P: PartialEq>(
    subtag1: &Option<P>,
    subtag2: &Option<P>,
    as_range1: bool,
    as_range2: bool,
) -> bool {
    (as_range1 && subtag1.is_none()) || (as_range2 && subtag2.is_none()) || subtag1 == subtag2
}

#[inline(always)]
fn matches(
    lid1: &LanguageIdentifier,
    lid2: &LanguageIdentifier,
    range1: bool,
    range2: bool,
) -> bool {
    ((range1 && lid1.language.is_unknown())
        || (range2 && lid2.language.is_unknown())
        || lid1.language == lid2.language)
        && subtag_matches(&lid1.script, &lid2.script, range1, range2)
        && subtag_matches(&lid1.region, &lid2.region, range1, range2)
        && ((range1 && lid1.variants.is_empty())
            || (range2 && lid2.variants.is_empty())
            || lid1.variants == lid2.variants)
}

pub fn negotiate_language<'a>(
    requested: Vec<&LanguageIdentifier>,
    available: Vec<&'a LanguageIdentifier>,
) -> Option<&'a LanguageIdentifier> {
    let find_match = |req: &LanguageIdentifier, self_as_range: bool, other_as_range: bool| {
        available
            .iter()
            .copied()
            .find(|locale| matches(locale, req, self_as_range, other_as_range))
    };

    for req in requested {
        // 1) Try to find a simple (case-insensitive) string match for the request.
        if let Some(locale) = find_match(req, false, false) {
            return Some(locale);
        }

        // 2) Try to match against the available locales treated as ranges.
        if let Some(locale) = find_match(req, true, false) {
            return Some(locale);
        }

        // Per Unicode TR35, 4.4 Locale Matching, we don't add likely subtags to
        // requested locales, so we'll skip it from the rest of the steps.
        if req.language.is_unknown() {
            continue;
        }

        let mut req = req.to_owned();

        // 3) Try to match against a maximized version of the requested locale
        if LOCALE_EXPANDER.maximize(&mut req) == TransformResult::Modified {
            if let Some(locale) = find_match(&req, true, false) {
                return Some(locale);
            }
        }

        // 4) Try to match against a variant as a range
        req.variants.clear();
        if let Some(locale) = find_match(&req, true, true) {
            return Some(locale);
        }

        // 5) Try to match against the likely subtag without region
        req.region = None;
        if LOCALE_EXPANDER.maximize(&mut req) == TransformResult::Modified {
            if let Some(locale) = find_match(&req, true, false) {
                return Some(locale);
            }
        }

        // 6) Try to match against a region as a range
        req.region = None;
        if let Some(locale) = find_match(&req, true, true) {
            return Some(locale);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_negotiate_fr_fr() {
        let requested = vec![
            LanguageIdentifier::from_str("fr-FR").unwrap(),
            LanguageIdentifier::from_str("en").unwrap(),
        ];
        let available = vec![
            LanguageIdentifier::from_str("en-US").unwrap(),
            LanguageIdentifier::from_str("fr-FR").unwrap(),
            LanguageIdentifier::from_str("en").unwrap(),
            LanguageIdentifier::from_str("fr").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "fr-FR"
        );
    }

    #[test]
    fn test_negotiate_fr() {
        let requested = vec![
            LanguageIdentifier::from_str("fr").unwrap(),
            LanguageIdentifier::from_str("en").unwrap(),
        ];
        let available = vec![
            LanguageIdentifier::from_str("en-US").unwrap(),
            LanguageIdentifier::from_str("fr-FR").unwrap(),
            LanguageIdentifier::from_str("en").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "fr-FR"
        );
    }

    #[test]
    fn test_negotiate_en_us() {
        let requested = vec![
            LanguageIdentifier::from_str("en").unwrap(),
            LanguageIdentifier::from_str("de").unwrap(),
        ];
        let available = vec![
            LanguageIdentifier::from_str("en-GB").unwrap(),
            LanguageIdentifier::from_str("en-US").unwrap(),
            LanguageIdentifier::from_str("de").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "en-US"
        );
    }

    #[test]
    fn test_zh() {
        let requested = vec![LanguageIdentifier::from_str("zh").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("zh-Hant").unwrap(),
            LanguageIdentifier::from_str("zh-Hans").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "zh-Hans"
        );
    }

    #[test]
    fn test_zh_cn() {
        let requested = vec![LanguageIdentifier::from_str("zh-CN").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("zh-Hant").unwrap(),
            LanguageIdentifier::from_str("zh-Hans").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "zh-Hans"
        );
    }

    #[test]
    fn test_es() {
        let requested = vec![LanguageIdentifier::from_str("es").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("es-419").unwrap(),
            LanguageIdentifier::from_str("es-ES").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "es-ES"
        );
    }

    #[test]
    fn test_es_us() {
        let requested = vec![LanguageIdentifier::from_str("es-US").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("es-ES").unwrap(),
            LanguageIdentifier::from_str("es-419").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "es-ES"
        );
    }

    #[test]
    fn test_es_419() {
        let requested = vec![LanguageIdentifier::from_str("es-419").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("es-ES").unwrap(),
            LanguageIdentifier::from_str("es-419").unwrap(),
        ];
        assert_eq!(
            negotiate_language(requested.iter().collect(), available.iter().collect())
                .unwrap()
                .to_string(),
            "es-419"
        );
    }

    #[test]
    fn test_negotiate_und() {
        let requested = vec![LanguageIdentifier::from_str("und").unwrap()];
        let available = vec![
            LanguageIdentifier::from_str("en-GB").unwrap(),
            LanguageIdentifier::from_str("en-US").unwrap(),
            LanguageIdentifier::from_str("de").unwrap(),
        ];

        assert!(
            negotiate_language(requested.iter().collect(), available.iter().collect()).is_none()
        )
    }

    #[test]
    fn test_negotiate_empty() {
        let requested = vec![];
        let available = vec![
            LanguageIdentifier::from_str("en-GB").unwrap(),
            LanguageIdentifier::from_str("en-US").unwrap(),
            LanguageIdentifier::from_str("de").unwrap(),
        ];

        assert!(
            negotiate_language(requested.iter().collect(), available.iter().collect()).is_none()
        )
    }

    #[test]
    fn test_negotiate_strategies() {
        struct Case<'a> {
            requested: &'a str,
            available: &'a [&'a str],
            expected: &'a str,
        }

        for case in [
            // 1) Try to find a simple (case-insensitive) string match for the request.
            // zh-hans matches zh-Hans
            Case {
                requested: "zh-hans",
                available: &["en-US", "zh-Hans", "zh-CN"],
                expected: "zh-Hans",
            },
            // 2) Try to match against the available locales treated as ranges.
            // Hans matches Hans
            Case {
                requested: "zh-Hans",
                available: &["en-US", "und-Hans", "zh-CN"],
                expected: "und-Hans",
            },
            // 3. maximized request
            // zh is maximized to zh-Hans-CN and und-Hans matches
            Case {
                requested: "zh",
                available: &["en-US", "und-Hans", "und-CN"],
                expected: "und-Hans",
            },
            // 4. variant as range
            // zh-Hans-CN matches zh-Hans-CN
            Case {
                requested: "zh-Hans-CN-variant1",
                available: &["en-US", "zh-Hans-CN-variant2"],
                expected: "zh-Hans-CN-variant2",
            },
            // 5. maximize without region
            // zh-Hans-TW => zh-Hans-CN matches und-CN
            Case {
                requested: "zh-Hans-TW",
                available: &["en-US", "und-CN"],
                expected: "und-CN",
            },
            // 6. region as range
            // zh matches zh
            Case {
                requested: "zh-Hans-CN",
                available: &["en-US", "zh-TW"],
                expected: "zh-TW",
            },
        ] {
            let requested = vec![LanguageIdentifier::from_str(case.requested).unwrap()];
            let available = case
                .available
                .iter()
                .map(|locale| LanguageIdentifier::from_str(locale).unwrap())
                .collect::<Vec<_>>();

            assert_eq!(
                negotiate_language(requested.iter().collect(), available.iter().collect())
                    .unwrap()
                    .to_string(),
                case.expected
            );
        }
    }
}
