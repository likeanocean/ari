use std::fmt::{Display, Formatter};
use std::time::Duration;


pub struct FormattedDuration(pub Duration);

impl Display for FormattedDuration {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let mut t = self.0.as_secs();

        let seconds = t % 60;
        t /= 60;

        let minutes = t % 60;
        t /= 60;

        let hours = t % 24;
        t /= 24;

        match t {
            t if t > 0 => write!(formatter, "{}d {:02}:{:02}:{:02}", t, hours, minutes, seconds),
            _ => write!(formatter, "{:02}:{:02}:{:02}", hours, minutes, seconds),
        }
    }
}


pub struct HumanDuration(pub Duration);

impl Display for HumanDuration {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let seconds = self.0.as_secs();
        let alternate = formatter.alternate();

        macro_rules! try_unit {
            ($seconds: expr, $singular: expr, $plural: expr, $shorthand: expr) => {
                let count = seconds / $seconds;

                match (alternate, count) {
                    (true, 0) => {},
                    (true, 1) => return write!(formatter, "{}{}", count, $shorthand),
                    (true, _) => return write!(formatter, "{}{}", count, $shorthand),
                    (false, 0) => {},
                    (false, 1) => return write!(formatter, "{} {}", count, $singular),
                    (false, _) => return write!(formatter, "{} {}", count, $plural),
                }
            };
        }

        try_unit!(365 * 24 * 60 * 60, "year", "years", "y");
        try_unit!(7 * 24 * 60 * 60, "week", "weeks", "w");
        try_unit!(24 * 60 * 60, "day", "days", "d");
        try_unit!(60 * 60, "hour", "hours", "h");
        try_unit!(60, "minute", "minutes", "m");
        try_unit!(1, "second", "seconds", "s");

        write!(formatter, "0{}", if alternate { "s" } else { " seconds" })
    }
}



#[derive(Clone, Copy, Debug)]
pub struct HumanBytes(pub u64);

impl Display for HumanBytes {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        const DEFAULT_PLACES: usize = 2;

        Display::fmt(&HumanDetailedBytes(self.0, DEFAULT_PLACES), formatter)
    }
}



#[derive(Clone, Copy, Debug)]
pub struct HumanDetailedBytes(pub u64, pub usize);

impl Display for HumanDetailedBytes {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let kib = 1024.0 as f64;
        let HumanDetailedBytes(bytes, places) = *self;

        match bytes as f64 {
            n if n >= kib.powf(6.0) => write!(formatter, "{:.*} EiB", places, n / kib.powf(6.0)),
            n if n >= kib.powf(5.0) => write!(formatter, "{:.*} PiB", places, n / kib.powf(5.0)),
            n if n >= kib.powf(4.0) => write!(formatter, "{:.*} TiB", places, n / kib.powf(4.0)),
            n if n >= kib.powf(3.0) => write!(formatter, "{:.*} GiB", places, n / kib.powf(3.0)),
            n if n >= kib.powf(2.0) => write!(formatter, "{:.*} MiB", places, n / kib.powf(2.0)),
            n if n >= kib => write!(formatter, "{:.*} KiB", places, n / kib),
            n => write!(formatter, "{:.*} B", 0, n),
        }
    }
}
