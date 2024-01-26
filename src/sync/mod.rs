use parking_lot::{Condvar, Mutex};
use std::num::Wrapping;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub trait ResetEvent {
    /// sets the state of the event to nonsignaled, causing threads to block.
    ///
    /// returns true if the operation succeeded.
    fn reset(&self);

    /// sets the state of the event to signaled, allowing one or more waiting threads to proceed.
    ///
    /// returns true if the operation succeeded.
    fn set(&self);

    /// wait for a signal.
    fn wait(&self);

    /// returns true if this wait received a signal, otherwise false.
    fn wait_until(&self, instant: Instant) -> bool;

    /// returns true if this wait received a signal, otherwise false.
    fn wait_duration(&self, duration: Duration) -> bool {
        self.wait_until(Instant::now() + duration)
    }

    /// returns true if this wait received a signal, otherwise false.
    fn wait_ms(&self, milliseconds: u64) -> bool {
        self.wait_duration(Duration::from_millis(milliseconds))
    }
}

/// a thread waits for a signal by calling waitone on the autoresetevent. if the autoresetevent is in the non-signaled
/// state, the thread blocks, waiting for the thread that currently controls the resource to signal that the resource is
/// available by calling set.
///
/// calling set signals autoresetevent to release a waiting thread. autoresetevent remains signaled until a single
/// waiting thread is released, and then automatically returns to the non-signaled state. if no threads are waiting, the
/// state remains signaled indefinitely.
///
/// if a thread calls waitone while the autoresetevent is in the signaled state, the thread does not block. the
/// autoresetevent releases the thread immediately and returns to the non-signaled state.
pub struct AutoResetEvent {
    value: Mutex<bool>,
    condition: Condvar,
}

impl AutoResetEvent {
    pub fn new(signalled: bool) -> AutoResetEvent {
        AutoResetEvent {
            value: Mutex::new(signalled),
            condition: Condvar::new(),
        }
    }
}

impl ResetEvent for AutoResetEvent {
    fn reset(&self) {
        *self.value.lock() = false;
    }

    fn set(&self) {
        *self.value.lock() = true;
        self.condition.notify_one();
    }

    fn wait(&self) {
        let mut value = self.value.lock();

        while !*value {
            self.condition.wait(&mut value);
        }

        *value = false;
    }

    fn wait_until(&self, instant: Instant) -> bool {
        let mut value = self.value.lock();

        while !*value {
            if self.condition.wait_until(&mut value, instant).timed_out() {
                return false;
            }
        }

        *value = false;
        true
    }
}

/// manualresetevent allows threads to communicate with each other by signaling. typically, this communication concerns
/// a task which one thread must complete before other threads can proceed.
///
/// when a thread begins an activity that must complete before other threads proceed, it calls reset to put
/// manualresetevent in the non-signaled state. this thread can be thought of as controlling the manualresetevent.
/// threads that call waitone on the manualresetevent will block, awaiting the signal. when the controlling thread
/// completes the activity, it calls set to signal that the waiting threads can proceed. all waiting threads are
/// released.
///
/// once it has been signaled, manualresetevent remains signaled until it is manually reset. that is, calls to waitone
/// return immediately.
///
/// you can control the initial state of a manualresetevent by passing a boolean value to the constructor, true if the
/// initial state is signaled and false otherwise.
pub struct ManualResetEvent {
    value: Mutex<ManualResetEventData>,
    condition: Condvar,
}

impl ManualResetEvent {
    pub fn new(signalled: bool) -> ManualResetEvent {
        let id = Wrapping(0);

        ManualResetEvent {
            value: Mutex::new(ManualResetEventData { signalled, id }),
            condition: Condvar::new(),
        }
    }
}

struct ManualResetEventData {
    id: Wrapping<usize>,
    signalled: bool,
}

impl ResetEvent for ManualResetEvent {
    fn reset(&self) {
        self.value.lock().signalled = false;
    }

    fn set(&self) {
        let mut value = self.value.lock();

        value.id += Wrapping(1);
        value.signalled = true;

        self.condition.notify_all();
    }

    fn wait(&self) {
        let mut value = self.value.lock();
        let id = value.id;

        while !value.signalled && value.id == id {
            self.condition.wait(&mut value);
        }
    }

    fn wait_until(&self, instant: Instant) -> bool {
        let mut value = self.value.lock();
        let id = value.id;

        while !value.signalled && value.id == id {
            if self.condition.wait_until(&mut value, instant).timed_out() {
                return false;
            }
        }

        true
    }
}

/// an atomic boolean that always uses acquire and release semantics.
#[derive(Debug)]
pub struct AtomicArBool {
    value: AtomicBool,
}

impl AtomicArBool {
    pub const fn new(value: bool) -> AtomicArBool {
        AtomicArBool {
            value: AtomicBool::new(value),
        }
    }

    pub fn get(&self) -> bool {
        self.value.load(Ordering::Acquire)
    }

    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::Release)
    }
}

impl Default for AtomicArBool {
    fn default() -> AtomicArBool {
        AtomicArBool {
            value: AtomicBool::default(),
        }
    }
}

/// an atomic boolean that uses relaxed semantics.
#[derive(Debug)]
pub struct AtomicRelaxedBool {
    value: AtomicBool,
}

impl AtomicRelaxedBool {
    pub const fn new(value: bool) -> AtomicRelaxedBool {
        AtomicRelaxedBool {
            value: AtomicBool::new(value),
        }
    }

    pub fn get(&self) -> bool {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::Relaxed)
    }
}

impl Default for AtomicRelaxedBool {
    fn default() -> AtomicRelaxedBool {
        AtomicRelaxedBool {
            value: AtomicBool::default(),
        }
    }
}
