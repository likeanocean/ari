use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// the number of nanoseconds in a microsecond.
pub const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;

/// the number of nanoseconds in a millisecond.
pub const NANOSECONDS_PER_MILLISECOND: u64 = 1_000_000;

/// the number of nanoseconds in seconds.
pub const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

/// the number of microseconds per second.
pub const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

/// the number of milliseconds per second.
pub const MILLISECONDS_PER_SECOND: u64 = 1_000;

/// the number of seconds in a minute.
pub const SECONDS_PER_MINUTE: u64 = 60;

/// the number of seconds in an hour.
pub const SECONDS_PER_HOUR: u64 = 3_600;

/// the number of (non-leap) seconds in days.
pub const SECONDS_PER_DAY: u64 = 86_400;

/// the number of (non-leap) seconds in a week.
pub const SECONDS_PER_WEEK: u64 = 604_800;

#[derive(Debug, Clone, Copy)]
pub struct Stopwatch {
    elapsed: Duration, // the recorded elapsed duration before the most recent stopwatch start.
    started: Option<Instant>, // the instant at which it was started. when none, stopwatch is not running.
}

impl Stopwatch {
    pub fn new() -> Stopwatch {
        Stopwatch {
            elapsed: Duration::default(),
            started: None,
        }
    }

    pub fn start(&mut self) {
        if self.started == None {
            self.started = Some(Instant::now());
        }
    }

    pub fn started() -> Stopwatch {
        let mut stopwatch = Stopwatch::new();

        stopwatch.start();
        stopwatch
    }

    pub fn stop(&mut self) {
        if self.started != None {
            self.elapsed = self.elapsed();
            self.started = None;
        }
    }

    pub fn reset(&mut self) -> Duration {
        let elapsed = self.elapsed();

        self.elapsed = Duration::default();
        self.started = self.started.map(|_| Instant::now());

        elapsed
    }

    pub fn restart(&mut self) {
        self.reset();
        self.start();
    }

    pub fn running(&self) -> bool {
        self.started != None
    }

    pub fn elapsed(&self) -> Duration {
        match self.started {
            Some(started) => started.elapsed() + self.elapsed,
            None => self.elapsed,
        }
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.elapsed().as_secs()
    }

    pub fn elapsed_milliseconds(&self) -> u128 {
        self.elapsed().as_millis()
    }

    pub fn elapsed_microseconds(&self) -> u128 {
        self.elapsed().as_micros()
    }

    pub fn elapsed_nanoseconds(&self) -> u128 {
        self.elapsed().as_nanos()
    }
}

impl Default for Stopwatch {
    fn default() -> Stopwatch {
        Stopwatch::new()
    }
}

impl Display for Stopwatch {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:?}", self.elapsed())
    }
}

/// a simple fps clock / fps counter.
///
/// # examples.
///
/// ```
/// # use ari::time::FpsClock;
///
/// let mut clock = FpsClock::new();
///
/// for _ in 0..100 {
///     clock.update();
/// }
///
/// assert_eq!(clock.fps() > 0, true);
/// ```
pub struct FpsClock {
    queue: VecDeque<Instant>,
    frames: Arc<AtomicUsize>,
}

impl FpsClock {
    pub fn new() -> FpsClock {
        FpsClock {
            queue: VecDeque::with_capacity(1024),
            frames: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let previous = now - Duration::from_secs(1);

        self.queue.push_back(now);

        while self.queue.front().map(|x| *x < previous).unwrap_or(false) {
            self.queue.pop_front();
        }

        self.frames.store(self.queue.len(), Ordering::Relaxed);
    }

    pub fn client(&self) -> Fps {
        Fps {
            frames: self.frames.clone(),
        }
    }

    pub fn fps(&self) -> usize {
        self.frames.load(Ordering::Relaxed)
    }
}

impl Debug for FpsClock {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct("FpsClock")
            .field("fps", &self.fps())
            .finish()
    }
}

/// reads the fps value of some `FpsClock`.
///
/// # examples.
///
/// ```
/// # use ari::time::{Fps, FpsClock};
///
/// let mut clock = FpsClock::new();
///
/// for _ in 0..100 {
///     clock.update();
/// }
///
/// let fps = clock.client();
///
/// assert_eq!(fps.value() > 0, true);
/// ```
pub struct Fps {
    frames: Arc<AtomicUsize>,
}

impl Fps {
    pub fn value(&self) -> usize {
        self.frames.load(Ordering::Relaxed)
    }
}

impl Debug for Fps {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct("Fps")
            .field("fps", &self.value())
            .finish()
    }
}
