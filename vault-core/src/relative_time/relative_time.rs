use crate::{intl, runtime, types::TimeMillis};

use super::{RelativeTimeDiff, RelativeTimeModifier};

#[derive(Clone, Debug)]
pub struct RelativeTime {
    pub value: TimeMillis,
    pub display: String,
    pub next_update: Option<TimeMillis>,
}

impl RelativeTime {
    pub fn new(
        runtime: &runtime::BoxRuntime,
        value: TimeMillis,
        with_modifier: bool,
        intl_service: &intl::IntlService,
    ) -> Self {
        let now = runtime.now();

        let duration = value - now;

        let (diff, next_update) = RelativeTimeDiff::from_duration(duration);
        let modifier = RelativeTimeModifier::from_duration(duration, with_modifier);

        let display = format_relative_time(&diff, &modifier, intl_service);

        let next_update = next_update.map(|d| now + d);

        Self {
            value,
            display,
            next_update,
        }
    }
}

pub fn format_relative_time(
    diff: &RelativeTimeDiff,
    modifier: &RelativeTimeModifier,
    intl_service: &intl::IntlService,
) -> String {
    use RelativeTimeDiff::*;
    use RelativeTimeModifier::*;

    let (diff, value) = match diff {
        FewSeconds => ("few_seconds", 0),
        Seconds(n) => ("seconds", *n),
        Minute => ("minutes", 1),
        Minutes(n) => ("minutes", *n),
        Hour => ("hours", 1),
        Hours(n) => ("hours", *n),
        Day => ("days", 1),
        Days(n) => ("days", *n),
        Month => ("months", 1),
        Months(n) => ("months", *n),
        Year => ("years", 1),
        Years(n) => ("years", *n),
    };

    let modifier = match modifier {
        Past => "past",
        Future => "future",
        None => "other",
    };

    intl::format_message!(
        intl_service,
        "core.relative_time",
        "Relative time message shown for past, future, and neutral values.",
        r###"{modifier, select,
  past {{diff, select,
    few_seconds {a few seconds ago}
    seconds {{value, plural,
      one {a second ago}
      other {# seconds ago}
    }}
    minutes {{value, plural,
      one {a minute ago}
      other {# minutes ago}
    }}
    hours {{value, plural,
      one {an hour ago}
      other {# hours ago}
    }}
    days {{value, plural,
      one {a day ago}
      other {# days ago}
    }}
    months {{value, plural,
      one {a month ago}
      other {# months ago}
    }}
    years {{value, plural,
      one {a year ago}
      other {# years ago}
    }}
    other {}
  }}
  future {{diff, select,
    few_seconds {in a few seconds}
    seconds {{value, plural,
      one {in a second}
      other {in # seconds}
    }}
    minutes {{value, plural,
      one {in a minute}
      other {in # minutes}
    }}
    hours {{value, plural,
      one {in an hour}
      other {in # hours}
    }}
    days {{value, plural,
      one {in a day}
      other {in # days}
    }}
    months {{value, plural,
      one {in a month}
      other {in # months}
    }}
    years {{value, plural,
      one {in a year}
      other {in # years}
    }}
    other {}
  }}
  other {{diff, select,
    few_seconds {a few seconds}
    seconds {{value, plural,
      one {a second}
      other {# seconds}
    }}
    minutes {{value, plural,
      one {a minute}
      other {# minutes}
    }}
    hours {{value, plural,
      one {an hour}
      other {# hours}
    }}
    days {{value, plural,
      one {a day}
      other {# days}
    }}
    months {{value, plural,
      one {a month}
      other {# months}
    }}
    years {{value, plural,
      one {a year}
      other {# years}
    }}
    other {}
  }}
}"###,
        &[
            ("modifier", intl::FormatValue::String(modifier)),
            ("diff", intl::FormatValue::String(diff)),
            ("value", intl::FormatValue::Integer(value))
        ]
    )
}

#[cfg(test)]
mod tests {
    use std::{ops::Neg, sync::Arc};

    use chrono::Duration;

    use crate::{
        intl,
        secure_storage::{MemorySecureStorage, SecureStorageService},
        store,
    };

    use super::*;

    #[test]
    fn test_format_relative_time() {
        fn case(duration_ms: i64) -> (String, String, String) {
            let duration = Duration::milliseconds(duration_ms);
            let intl_service = intl::IntlService::new(
                Arc::new(SecureStorageService::new(Box::new(
                    MemorySecureStorage::new(),
                ))),
                Arc::new(store::Store::new(store::State::default())),
            );

            (
                format_relative_time(
                    &RelativeTimeDiff::from_duration(duration).0,
                    &RelativeTimeModifier::from_duration(duration, false),
                    &intl_service,
                ),
                format_relative_time(
                    &RelativeTimeDiff::from_duration(duration.neg()).0,
                    &RelativeTimeModifier::from_duration(duration.neg(), true),
                    &intl_service,
                ),
                format_relative_time(
                    &RelativeTimeDiff::from_duration(duration).0,
                    &RelativeTimeModifier::from_duration(duration, true),
                    &intl_service,
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
