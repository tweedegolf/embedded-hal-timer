#![no_std]

use core::{error::Error, fmt::Display};

/// A timer that can be started from 0 and keeps track of the time until it overflows.
pub trait Timer {
    /// Start or restart the timer at 0.
    /// 
    /// The elapsed time is undefined before this function has been called.
    fn start(&mut self);

    /// Get the amount of ticks per second.
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

    /// The (inclusive) maximum number of ticks that can happen before the overflow occurs.
    fn max_ticks(&self) -> u64;
    /// The (inclusive) maximum number of nanoseconds that can happen before the overflow occurs.
    fn max_nanos(&self) -> u64;
    /// The (inclusive) maximum number of microseconds that can happen before the overflow occurs.
    fn max_micros(&self) -> u64;
    /// The (inclusive) maximum number of milliseconds that can happen before the overflow occurs.
    fn max_millis(&self) -> u64;
    /// The (inclusive) maximum number of seconds that can happen before the overflow occurs.
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
