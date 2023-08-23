use crate::relative_time::{RelativeTimeDiff, RelativeTimeModifier};

pub trait Locale {
    fn relative_time(&self, diff: &RelativeTimeDiff, modifier: &RelativeTimeModifier) -> String;
}

pub type BoxLocale = Box<dyn Locale + Send + Sync>;

pub fn get_locale(name: &str) -> Option<BoxLocale> {
    match name {
        "en" => Some(Box::new(LocaleEn {})),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub struct LocaleEn {}

impl Locale for LocaleEn {
    fn relative_time(&self, diff: &RelativeTimeDiff, modifier: &RelativeTimeModifier) -> String {
        use RelativeTimeDiff::*;
        use RelativeTimeModifier::*;

        let s = match diff {
            FewSeconds => format!("a few seconds"),
            Seconds(n) => format!("{} seconds", n),
            Minute => format!("a minute"),
            Minutes(n) => format!("{} minutes", n),
            Hour => format!("an hour"),
            Hours(n) => format!("{} hours", n),
            Day => format!("a day"),
            Days(n) => format!("{} days", n),
            Month => format!("a month"),
            Months(n) => format!("{} months", n),
            Year => format!("a year"),
            Years(n) => format!("{} years", n),
        };

        match modifier {
            None => s,
            Past => format!("{} ago", s),
            Future => format!("in {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Neg;

    use chrono::Duration;

    use crate::{
        locale::{BoxLocale, LocaleEn},
        relative_time::{RelativeTimeDiff, RelativeTimeModifier},
    };

    #[test]
    fn test_locale_relative_time() {
        fn case(duration_ms: i64) -> (String, String, String) {
            let duration = Duration::milliseconds(duration_ms);
            let locale: BoxLocale = Box::new(LocaleEn::default());

            (
                locale.relative_time(
                    &RelativeTimeDiff::from_duration(duration).0,
                    &RelativeTimeModifier::from_duration(duration, false),
                ),
                locale.relative_time(
                    &RelativeTimeDiff::from_duration(duration.neg()).0,
                    &RelativeTimeModifier::from_duration(duration.neg(), true),
                ),
                locale.relative_time(
                    &RelativeTimeDiff::from_duration(duration).0,
                    &RelativeTimeModifier::from_duration(duration, true),
                ),
            )
        }

        assert_eq!(
            case(0),
            (
                "a few seconds".into(),
                "a few seconds ago".into(),
                "a few seconds ago".into()
            )
        );
        assert_eq!(
            case(44499),
            (
                "a few seconds".into(),
                "a few seconds ago".into(),
                "in a few seconds".into()
            )
        );
        assert_eq!(
            case(44500),
            (
                "a minute".into(),
                "a minute ago".into(),
                "in a minute".into()
            )
        );
        assert_eq!(
            case(89999),
            (
                "a minute".into(),
                "a minute ago".into(),
                "in a minute".into()
            )
        );
        assert_eq!(
            case(90000),
            (
                "2 minutes".into(),
                "2 minutes ago".into(),
                "in 2 minutes".into()
            )
        );
        assert_eq!(
            case(2669999),
            (
                "44 minutes".into(),
                "44 minutes ago".into(),
                "in 44 minutes".into()
            )
        );
        assert_eq!(
            case(2670000),
            ("an hour".into(), "an hour ago".into(), "in an hour".into())
        );
        assert_eq!(
            case(5399999),
            ("an hour".into(), "an hour ago".into(), "in an hour".into())
        );
        assert_eq!(
            case(5400000),
            ("2 hours".into(), "2 hours ago".into(), "in 2 hours".into())
        );
        assert_eq!(
            case(77399999),
            (
                "21 hours".into(),
                "21 hours ago".into(),
                "in 21 hours".into()
            )
        );
        assert_eq!(
            case(77400000),
            ("a day".into(), "a day ago".into(), "in a day".into())
        );
        assert_eq!(
            case(129599999),
            ("a day".into(), "a day ago".into(), "in a day".into())
        );
        assert_eq!(
            case(129600000),
            ("2 days".into(), "2 days ago".into(), "in 2 days".into())
        );
        assert_eq!(
            case(2203199999),
            ("25 days".into(), "25 days ago".into(), "in 25 days".into())
        );
        assert_eq!(
            case(2203200000),
            ("a month".into(), "a month ago".into(), "in a month".into())
        );
        assert_eq!(
            case(3974399999),
            ("a month".into(), "a month ago".into(), "in a month".into())
        );
        assert_eq!(
            case(3974400000),
            (
                "2 months".into(),
                "2 months ago".into(),
                "in 2 months".into()
            )
        );
        assert_eq!(
            case(27647999999),
            (
                "10 months".into(),
                "10 months ago".into(),
                "in 10 months".into()
            )
        );
        assert_eq!(
            case(27648000000),
            ("a year".into(), "a year ago".into(), "in a year".into())
        );
        assert_eq!(
            case(46051199999),
            ("a year".into(), "a year ago".into(), "in a year".into())
        );
        assert_eq!(
            case(46051200000),
            ("2 years".into(), "2 years ago".into(), "in 2 years".into())
        );
    }
}
