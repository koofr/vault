use std::pin::Pin;

use futures::{AsyncRead, AsyncWrite};

pub type BoxAsyncRead = Pin<Box<dyn AsyncRead + Send + Sync + 'static>>;

pub type BoxAsyncWrite = Pin<Box<dyn AsyncWrite + Send + Sync + 'static>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status<E: Clone> {
    Initial,
    Loading,
    Loaded,
    Reloading,
    Error { error: E },
}

impl<E: Clone> Default for Status<E> {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RemainingTime {
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl RemainingTime {
    pub fn from_seconds(total_seconds: f64) -> Self {
        let mut total = total_seconds;

        let days = (total / (24.0 * 3600.0)).floor() as u32;
        total %= 24.0 * 3600.0;

        let hours = (total / 3600.0).floor() as u32;
        total %= 3600.0;

        let minutes = (total / 60.0).floor() as u32;
        total %= 60.0;

        let seconds = total.ceil() as u32;

        RemainingTime {
            days,
            hours,
            minutes,
            seconds,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SizeInfo {
    Exact(i64),
    Estimate(i64),
    Unknown,
}

impl SizeInfo {
    pub fn exact_or_estimate(&self) -> Option<i64> {
        match self {
            Self::Exact(size) => Some(*size),
            Self::Estimate(size) => Some(*size),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RemainingTime;

    #[test]
    fn test_remaining_time_from_seconds() {
        let remaining_time = RemainingTime::from_seconds(50.0 * 3600.0 + 45.0 * 60.0 + 30.0 + 0.7);

        assert_eq!(
            remaining_time,
            RemainingTime {
                days: 2,
                hours: 2,
                minutes: 45,
                seconds: 31,
            }
        )
    }
}
