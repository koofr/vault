use std::time::Duration;

#[derive(Copy, Clone)]
enum SizeUnit {
    B = 0,
    KB,
    MB,
    GB,
    TB,
}

impl SizeUnit {
    fn next(self) -> Option<Self> {
        match self {
            Self::B => Some(Self::KB),
            Self::KB => Some(Self::MB),
            Self::MB => Some(Self::GB),
            Self::GB => Some(Self::TB),
            Self::TB => None,
        }
    }
}

pub fn size_display(bytes: i64) -> String {
    let mut unit = SizeUnit::B;
    let mut size = bytes as f64;

    loop {
        match unit.next() {
            Some(next) if size >= 1024.0 => {
                unit = next;
                size /= 1024.0;
            }
            _ => break,
        }
    }

    size = (size * 10.0).round() / 10.0;

    match unit {
        SizeUnit::B => format!("{} B", size),
        SizeUnit::KB => format!("{} KB", size),
        SizeUnit::MB => format!("{} MB", size),
        SizeUnit::GB => format!("{} GB", size),
        SizeUnit::TB => format!("{} TB", size),
    }
}

pub fn size_of_display(bytes_current: i64, bytes_total: i64) -> String {
    let mut unit = SizeUnit::B;
    let mut size_current = bytes_current as f64;
    let mut size_total = bytes_total as f64;

    loop {
        match unit.next() {
            Some(next) if size_total >= 1024.0 => {
                unit = next;
                size_current /= 1024.0;
                size_total /= 1024.0;
            }
            _ => break,
        }
    }

    size_current = (size_current * 10.0).round() / 10.0;
    size_total = (size_total * 10.0).round() / 10.0;

    match unit {
        SizeUnit::B => format!("{} / {} B", size_current, size_total),
        SizeUnit::KB => format!("{} / {} KB", size_current, size_total),
        SizeUnit::MB => format!("{} / {} MB", size_current, size_total),
        SizeUnit::GB => format!("{} / {} GB", size_current, size_total),
        SizeUnit::TB => format!("{} / {} TB", size_current, size_total),
    }
}

pub fn speed_display_bytes_per_second(bytes_per_second: i64) -> String {
    format!("{}/s", size_display(bytes_per_second))
}

pub fn speed_display_bytes_duration(bytes: i64, duration: Duration) -> String {
    if duration.is_zero() {
        speed_display_bytes_per_second(0)
    } else {
        let bytes_per_second = (bytes as f64 / duration.as_secs_f64()) as i64;

        speed_display_bytes_per_second(bytes_per_second)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::file_size::{size_display, size_of_display, speed_display_bytes_duration};

    #[test]
    fn test_size_display() {
        assert_eq!(size_display(0), "0 B");
        assert_eq!(size_display(1), "1 B");
        assert_eq!(size_display(1023), "1023 B");
        assert_eq!(size_display(1024), "1 KB");
        assert_eq!(size_display(1075), "1 KB");
        assert_eq!(size_display(1076), "1.1 KB");
        assert_eq!(size_display(1024 * 1024 - 52), "1023.9 KB");
        assert_eq!(size_display(1024 * 1024 - 51), "1024 KB");
        assert_eq!(size_display(1024 * 1024 - 1), "1024 KB");
        assert_eq!(size_display(1024 * 1024), "1 MB");
        assert_eq!(size_display(1024 * 1024 * 1024), "1 GB");
        assert_eq!(size_display(1024 * 1024 * 1024 * 1024), "1 TB");
        assert_eq!(size_display(1024 * 1024 * 1024 * 1024 * 1024), "1024 TB");
        assert_eq!(
            size_display(1024 * 1024 * 1024 * 1024 * 1024 * 2),
            "2048 TB"
        );
    }

    #[test]
    fn test_size_of_display() {
        assert_eq!(size_of_display(0, 0), "0 / 0 B");
        assert_eq!(size_of_display(0, 1), "0 / 1 B");
        assert_eq!(size_of_display(0, 1023), "0 / 1023 B");
        assert_eq!(size_of_display(1023, 1023), "1023 / 1023 B");
        assert_eq!(size_of_display(0, 1024), "0 / 1 KB");
        assert_eq!(size_of_display(51, 1024), "0 / 1 KB");
        assert_eq!(size_of_display(52, 1024), "0.1 / 1 KB");
        assert_eq!(size_of_display(972, 1024), "0.9 / 1 KB");
        assert_eq!(size_of_display(973, 1024), "1 / 1 KB");
        assert_eq!(size_of_display(1, 1076), "0 / 1.1 KB");
        assert_eq!(size_of_display(1, 1024 * 1024 - 52), "0 / 1023.9 KB");
        assert_eq!(size_of_display(1, 1024 * 1024 - 51), "0 / 1024 KB");
        assert_eq!(size_of_display(1, 1024 * 1024 - 1), "0 / 1024 KB");
        assert_eq!(size_of_display(1, 1024 * 1024), "0 / 1 MB");
        assert_eq!(size_of_display(1, 1024 * 1024 * 1024), "0 / 1 GB");
        assert_eq!(size_of_display(1, 1024 * 1024 * 1024 * 1024), "0 / 1 TB");
        assert_eq!(
            size_of_display(1, 1024 * 1024 * 1024 * 1024 * 1024),
            "0 / 1024 TB"
        );
        assert_eq!(
            size_of_display(1, 1024 * 1024 * 1024 * 1024 * 1024 * 2),
            "0 / 2048 TB"
        );
    }

    #[test]
    fn test_speed_display_bytes_duration() {
        assert_eq!(
            speed_display_bytes_duration(0, Duration::from_secs(10)),
            "0 B/s"
        );
        assert_eq!(
            speed_display_bytes_duration(100, Duration::from_secs(10)),
            "10 B/s"
        );
        assert_eq!(speed_display_bytes_duration(100, Duration::ZERO), "0 B/s");
    }
}
