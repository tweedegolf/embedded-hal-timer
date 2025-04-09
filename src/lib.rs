#![cfg_attr(not(test), no_std)]

#[cfg(feature = "embassy-stm32")]
pub mod impl_embassy_stm32;
#[cfg(feature = "embassy-time")]
pub mod impl_embassy_time;

/// The time has overflowed
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct OverflowError;

/// A timer that can be started from 0 and keeps track of the time until it overflows.
pub trait Timer {
    /// Start or restart the timer at 0.
    fn start(&mut self);

    #[cfg(feature = "ticks-api")]
    /// Get the amount of ticks per second.
    fn tickrate(&self) -> u32;
    #[cfg(feature = "ticks-api")]
    /// Return the number of elapsed ticks.
    fn elapsed_ticks(&self) -> Result<u32, OverflowError>;

    /// Return the number of elapsed microseconds, rounded down.
    fn elapsed_micros(&self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed milliseconds, rounded down.
    fn elapsed_millis(&self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed seconds, rounded down.
    fn elapsed_secs(&self) -> Result<u32, OverflowError>;

    #[cfg(feature = "max-api")]
    /// The (inclusive) maximum number of microseconds that can happen before the overflow occurs.
    fn max_micros(&self) -> u32;
    #[cfg(feature = "max-api")]
    /// The (inclusive) maximum number of milliseconds that can happen before the overflow occurs.
    fn max_millis(&self) -> u32;
    #[cfg(feature = "max-api")]
    /// The (inclusive) maximum number of seconds that can happen before the overflow occurs.
    fn max_secs(&self) -> u32;
    #[cfg(all(feature = "max-api", feature = "ticks-api"))]
    /// The (inclusive) maximum number of ticks that can happen before the overflow occurs.
    fn max_ticks(&self) -> u32;
}

/// An alarm that can be used to wait for a time to come.
#[allow(async_fn_in_trait)]
pub trait Alarm: Timer {
    #[cfg(feature = "ticks-api")]
    /// Wait until the timer reaches the alarm specified in ticks since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_ticks(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in microseconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_micros(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in milliseconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_millis(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in seconds since the timer has started.
    /// If the alarm is already reached, the function exits immediately.
    ///
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_secs(&mut self, value: u32) -> Result<(), OverflowError>;
}
