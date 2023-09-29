use std::{fmt::Display, pin::Pin};

use futures::{AsyncRead, AsyncWrite};

pub type BoxAsyncRead = Pin<Box<dyn AsyncRead + Send + Sync + 'static>>;

pub type BoxAsyncWrite = Pin<Box<dyn AsyncWrite + Send + Sync + 'static>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Status<E: std::error::Error + Clone + PartialEq> {
    Initial,
    Loading { loaded: bool },
    Loaded,
    Error { error: E, loaded: bool },
}

impl<E: std::error::Error + Clone + PartialEq> Status<E> {
    pub fn loaded(&self) -> bool {
        match self {
            Self::Initial => false,
            Self::Loading { loaded } => *loaded,
            Self::Loaded => true,
            Self::Error { loaded, .. } => *loaded,
        }
    }

    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Error { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl<E: std::error::Error + Clone + PartialEq> Default for Status<E> {
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

impl Display for RemainingTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.days > 0 {
            write!(f, "{}d ", self.days)?;
        }

        if self.hours > 0 {
            write!(f, "{}h ", self.hours)?;
        }

        if self.minutes > 0 {
            write!(f, "{}m ", self.minutes)?;
        }

        write!(f, "{}s", self.seconds | 0)?;

        Ok(())
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
