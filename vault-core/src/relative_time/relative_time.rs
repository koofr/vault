use chrono::Duration;

use crate::{locale::BoxLocale, runtime};

use super::{RelativeTimeDiff, RelativeTimeModifier};

#[derive(Clone, Debug)]
pub struct RelativeTime {
    pub value: i64,
    pub display: String,
    pub next_update: Option<i64>,
}

impl RelativeTime {
    pub fn new(
        runtime: &runtime::BoxRuntime,
        value: i64,
        locale: &BoxLocale,
        with_modifier: bool,
    ) -> Self {
        let now = runtime.now_ms();

        let duration = Duration::milliseconds(value - now);

        let (diff, next_update) = RelativeTimeDiff::from_duration(duration);
        let modifier = RelativeTimeModifier::from_duration(duration, with_modifier);

        let display = locale.relative_time(&diff, &modifier);

        let next_update = next_update.map(|d| now + d.num_milliseconds());

        Self {
            value,
            display,
            next_update,
        }
    }
}
