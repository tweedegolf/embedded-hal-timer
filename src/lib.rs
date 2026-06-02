#![no_std]
#![deny(missing_docs)]

//! Definition of [Timer] and [Alarm] traits.
//!
//! These traits are intended to be eventually included into embedded-hal & embedded-hal-async.
//! As such these traits are work in progress and the crate may receive breaking changes.

use core::{error::Error, fmt::Display};

/// A timer that can be started from 0 and keeps track of the time until it overflows.
///
/// This trait may be implemented directly on top of short running hardware timers
/// or on top of 'virtual' long running timers.
///
/// A driver using this trait should document the minimum required tick resolution and
/// the minimum required max time. The driver may choose to check if these invariants
/// are kept using the [`tickrate`](Self::tickrate) and the various [`max_*`](Self::max_ticks) functions.
pub trait Timer {
    /// Start or restart the timer at 0.
    ///
    /// The elapsed time is undefined before this function has been called.
    fn start(&mut self);

    /// Get the amount of ticks per second.
    ///
    /// This value should remain constant unless explicitly changed by the user of this trait out-of-band.
    fn tickrate(&self) -> u64;
    /// Return the number of elapsed ticks.
    fn elapsed_ticks(&self) -> Result<u64, OverflowError>;

    /// Return the number of elapsed nanoseconds, rounded down.
    fn elapsed_nanos(&self) -> Result<u64, OverflowError>;
    /// Return the number of elapsed microseconds, rounded down.
    fn elapsed_micros(&self) -> Result<u64, OverflowError>;
    /// Return the number of elapsed milliseconds, rounded down.
    fn elapsed_millis(&self) -> Result<u64, OverflowError>;
    /// Return the number of elapsed seconds, rounded down.
    fn elapsed_secs(&self) -> Result<u64, OverflowError>;

    /// The (inclusive) maximum number of ticks that can happen before the overflow occurs
    /// if [start](Self::start) were called now.
    ///
    /// This value is not necessarily constant as implementations built on top of continuously running
    /// timers will have shrinking amount of time left.
    fn max_ticks(&self) -> u64;
    /// The (inclusive) maximum number of nanoseconds that can happen before the overflow occurs
    /// if [start](Self::start) were called now.
    ///
    /// This value is not necessarily constant as implementations built on top of continuously running
    /// timers will have shrinking amount of time left.
    fn max_nanos(&self) -> u64;
    /// The (inclusive) maximum number of microseconds that can happen before the overflow occurs
    /// if [start](Self::start) were called now.
    ///
    /// This value is not necessarily constant as implementations built on top of continuously running
    /// timers will have shrinking amount of time left.
    fn max_micros(&self) -> u64;
    /// The (inclusive) maximum number of milliseconds that can happen before the overflow occurs
    /// if [start](Self::start) were called now.
    ///
    /// This value is not necessarily constant as implementations built on top of continuously running
    /// timers will have shrinking amount of time left.
    fn max_millis(&self) -> u64;
    /// The (inclusive) maximum number of seconds that can happen before the overflow occurs
    /// if [start](Self::start) were called now.
    ///
    /// This value is not necessarily constant as implementations built on top of continuously running
    /// timers will have shrinking amount of time left.
    fn max_secs(&self) -> u64;
}

/// An alarm that can be used to wait for a time to come.
#[allow(async_fn_in_trait)]
pub trait Alarm: Timer {
    /// Wait until the timer reaches the alarm specified in ticks since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_ticks(&mut self, value: u64) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in nanoseconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_nanos(&mut self, value: u64) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in microseconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_micros(&mut self, value: u64) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in milliseconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_millis(&mut self, value: u64) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in seconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_secs(&mut self, value: u64) -> Result<(), OverflowError>;
}

/// The timer has overflowed
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct OverflowError;

#[cfg(feature = "defmt")]
impl defmt::Format for OverflowError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "overflow")
    }
}

impl Display for OverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "overflow")
    }
}

impl Error for OverflowError {}
