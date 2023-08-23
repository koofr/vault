use chrono::Duration;

// 44.5 * 1000 (44.5 seconds)
const THRESHOLD_MINUTE: i64 = 44500;

// 1.5 * 60 * 1000 (1.5 minute)
const THRESHOLD_MINUTES: i64 = 90000;

// 44.5 * 60 * 1000 (44.5 minutes)
const THRESHOLD_HOUR: i64 = 2670000;

// 90 * 60 * 1000 (90 minutes)
const THRESHOLD_HOURS: i64 = 5400000;

// 21.5 * 60 * 60 * 1000 (21.5 hours)
const THRESHOLD_DAY: i64 = 77400000;

// 36 * 60 * 60 * 1000 (1.5 days)
const THRESHOLD_DAYS: i64 = 129600000;

// 25.5 * 24 * 60 * 60 * 1000 (25.5 days)
const THRESHOLD_MONTH: i64 = 2203200000;

// 46 * 24 * 60 * 60 * 1000 (1.5 months)
const THRESHOLD_MONTHS: i64 = 3974400000;

// 320 * 24 * 60 * 60 * 1000 (10.5 months)
const THRESHOLD_YEAR: i64 = 27648000000;

// 533 * 24 * 60 * 60 * 1000 (1.5 years)
const THRESHOLD_YEARS: i64 = 46051200000;

#[derive(Debug)]
pub enum RelativeTimeModifier {
    None,
    Past,
    Future,
}

impl RelativeTimeModifier {
    pub fn from_duration(duration: Duration, with_modifier: bool) -> Self {
        if with_modifier {
            if duration.is_zero() || duration.abs() != duration {
                RelativeTimeModifier::Past
            } else {
                RelativeTimeModifier::Future
            }
        } else {
            RelativeTimeModifier::None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RelativeTimeDiff {
    FewSeconds,
    Seconds(i64),
    Minute,
    Minutes(i64),
    Hour,
    Hours(i64),
    Day,
    Days(i64),
    Month,
    Months(i64),
    Year,
    Years(i64),
}

impl RelativeTimeDiff {
    pub fn from_duration(duration: Duration) -> (Self, Option<Duration>) {
        // from https://github.com/moment/moment/blob/000ac1800e620f770f4eb31b5ae908f6167b0ab2/src/lib/duration/humanize.js
        use RelativeTimeDiff::*;

        let abs_duration = duration.abs();
        let is_past = duration.abs() != duration || duration.is_zero();
        let duration = abs_duration;

        let milliseconds = duration.num_milliseconds();

        let next = |x: i64| Some(Duration::milliseconds(x));

        if milliseconds < THRESHOLD_MINUTE {
            return (
                FewSeconds,
                if is_past {
                    next(THRESHOLD_MINUTE - milliseconds)
                } else {
                    None
                },
            );
        }

        let minutes = (duration.num_seconds() as f32 / 60.0).round() as i64;

        if milliseconds < THRESHOLD_MINUTES {
            return (
                Minute,
                if is_past {
                    next(THRESHOLD_MINUTES - milliseconds)
                } else {
                    None
                },
            );
        }
        if milliseconds < THRESHOLD_HOUR {
            return (
                Minutes(minutes),
                if is_past {
                    next(
                        ((minutes * 60 + 30) * 1000 - milliseconds)
                            .min(THRESHOLD_HOUR - milliseconds + 1),
                    )
                } else {
                    None
                },
            );
        }

        let hours = (duration.num_minutes() as f32 / 60.0).round() as i64;

        if milliseconds < THRESHOLD_HOURS {
            return (
                Hour,
                if is_past {
                    next(THRESHOLD_HOURS - milliseconds)
                } else {
                    None
                },
            );
        }
        if milliseconds < THRESHOLD_DAY {
            return (
                Hours(hours),
                if is_past {
                    next(
                        ((hours * 60 + 30) * 60 * 1000 - milliseconds)
                            .min(THRESHOLD_DAY - milliseconds + 1),
                    )
                } else {
                    None
                },
            );
        }

        let days = (duration.num_hours() as f32 / 24.0).round() as i64;

        if milliseconds < THRESHOLD_DAYS {
            return (
                Day,
                if is_past {
                    next(THRESHOLD_DAYS - milliseconds)
                } else {
                    None
                },
            );
        }
        if milliseconds < THRESHOLD_MONTH {
            return (
                Days(days),
                if is_past {
                    next(
                        ((days * 24 + 12) * 60 * 60 * 1000 - milliseconds)
                            .min(THRESHOLD_MONTH - milliseconds + 1),
                    )
                } else {
                    None
                },
            );
        }

        let months = days_to_months(duration.num_days());

        if milliseconds < THRESHOLD_MONTHS {
            return (
                Month,
                if is_past {
                    next(THRESHOLD_MONTHS - milliseconds)
                } else {
                    None
                },
            );
        }
        if milliseconds < THRESHOLD_YEAR {
            return (
                Months(months),
                if is_past {
                    let next_month = [
                        0,
                        3974400000,
                        6652800000,
                        9244800000,
                        11836800000,
                        14515200000,
                        17107200000,
                        19785600000,
                        22377600000,
                        25056000000,
                        27648000000,
                    ][months as usize];

                    next((next_month - milliseconds).min(THRESHOLD_YEAR - milliseconds + 1))
                } else {
                    None
                },
            );
        }

        let years = (months as f32 / 12.0).round() as i64;

        if milliseconds < THRESHOLD_YEARS {
            return (
                Year,
                if is_past {
                    next(THRESHOLD_YEARS - milliseconds)
                } else {
                    None
                },
            );
        }

        (
            Years(years),
            if is_past {
                // not implemented for Years
                None
            } else {
                None
            },
        )
    }
}

fn days_to_months(days: i64) -> i64 {
    // from https://github.com/moment/moment/blob/000ac1800e620f770f4eb31b5ae908f6167b0ab2/src/lib/duration/bubble.js#L59
    // 400 years have 146097 days (taking into account leap year rules)
    // 400 years have 12 months === 4800
    ((days as f64 * 4800.0) / 146097.0).round() as i64
}

#[cfg(test)]
mod tests {
    use std::ops::Neg;

    use chrono::Duration;

    use super::RelativeTimeDiff;

    #[test]
    fn test_relative_time() {
        use RelativeTimeDiff::*;

        fn past(
            duration_ms: i64,
            expected_diff: RelativeTimeDiff,
            expected_next_update_ms: i64,
            expected_next_diff: RelativeTimeDiff,
        ) {
            let duration = Duration::milliseconds(duration_ms).neg();

            let (diff, next_update) = RelativeTimeDiff::from_duration(duration);
            let (last_diff, _) = RelativeTimeDiff::from_duration(
                duration - (next_update.unwrap() - Duration::milliseconds(1)),
            );
            let (next_diff, _) = RelativeTimeDiff::from_duration(duration - next_update.unwrap());

            assert_eq!(diff, expected_diff);
            assert_eq!(
                next_update.unwrap().num_milliseconds(),
                expected_next_update_ms
            );
            assert_eq!(last_diff, expected_diff);
            assert_eq!(next_diff, expected_next_diff);
        }

        past(0, FewSeconds, 44500, Minute);
        past(1, FewSeconds, 44499, Minute);
        past(44499, FewSeconds, 1, Minute);
        past(44500, Minute, 45500, Minutes(2));
        past(44501, Minute, 45499, Minutes(2));
        past(89999, Minute, 1, Minutes(2));
        past(90000, Minutes(2), 60000, Minutes(3));
        past(2669999, Minutes(44), 1, Hour);
        past(2670000, Hour, 2730000, Hours(2));
        past(5400000, Hours(2), 3600000, Hours(3));
        past(77399999, Hours(21), 1, Day);
        past(77400000, Day, 52200000, Days(2));
        past(129600000, Days(2), 86400000, Days(3));
        past(2203199999, Days(25), 1, Month);
        past(2203200000, Month, 1771200000, Months(2));
        past(3974400000, Months(2), 2678400000, Months(3));
        past(6652800000, Months(3), 2592000000, Months(4));
        past(9244800000, Months(4), 2592000000, Months(5));
        past(11836800000, Months(5), 2678400000, Months(6));
        past(14515200000, Months(6), 2592000000, Months(7));
        past(17107200000, Months(7), 2678400000, Months(8));
        past(19785600000, Months(8), 2592000000, Months(9));
        past(22377600000, Months(9), 2678400000, Months(10));
        past(25056000000, Months(10), 2592000000, Year);
        past(27648000000, Year, 18403200000, Years(2));
        assert_eq!(
            RelativeTimeDiff::from_duration(Duration::milliseconds(-46051200000)),
            (Years(2), None)
        );

        assert_eq!(
            RelativeTimeDiff::from_duration(Duration::milliseconds(44500)),
            (Minute, None)
        );
    }
}
