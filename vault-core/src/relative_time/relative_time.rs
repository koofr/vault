use crate::{locale::BoxLocale, runtime, types::TimeMillis};

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
        locale: &BoxLocale,
        with_modifier: bool,
    ) -> Self {
        let now = runtime.now();

        let duration = value - now;

        let (diff, next_update) = RelativeTimeDiff::from_duration(duration);
        let modifier = RelativeTimeModifier::from_duration(duration, with_modifier);

        let display = locale.relative_time(&diff, &modifier);

        let next_update = next_update.map(|d| now + d);

        Self {
            value,
            display,
            next_update,
        }
    }
}
