# Embedded-hal `Timer` and `Alarm`

Time handling in `embedded-hal` is limited to simple delays.
While a full all-encompasing time system like `embassy-time` is likely out of scope for the embedded-hal project, it would be helpful if some of the gaps are plugged.

This RFC is made up of two parts. The main proposal is a `Timer` trait for measuring the time between two or more points, intended for short durations.
Using `Timer` as a monotonic timer for the time since startup is a valid usecase, but is not the main target.
Additionally an `Alarm` trait is proposed that makes the `Timer` easier to use in some async contexts (instead of having to juggle both a `Timer` and a `DelayNs`).

If this RFC were accepted as is, there'd be three traits that have something to do with time:
- `DelayNs` (sync and async, already exists): Wait for a specified amount of time
- `Timer`: Start a running time at 0, and query how long it has been running
- `Alarm`: Wait until a `Timer` value has been reached

The traits have been implemented as a crate here: https://github.com/tweedegolf/embedded-hal-timer

## `Timer`

```rust
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
```

In its simplest form the user can start (and restart) the timer and get its elapsed time since the start.
The returned elapsed time is rounded down, so the contract is that *at least* this amount of time has passed. This is congruent with `DelayNs` and other timers in the ecosystem.

The user can use the ticks value instead of the fixed time values (e.g. millis) if the timer is used in a situation where high precision is required.

To be able to predict failure, users can query the max values that the timer supports. A driver that requires the max to be above a minimal threshold would do good to assert/check the value in some init stage.

A timer that can't detect overflows can't implement this trait.

## `Alarm`

```rust
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
```

The alarm allows the user to wait until the timer reaches a give value.

The proposal is to add only an async version of this trait. A blocking version could be made, but that's just a while loop that blocks on the timer which is trivial for any user of the `Timer` trait to already do.
This is similar to the `Wait` trait which also only has an async version.

Probably the most contentious part of the `Alarm` trait is that it inherits from the `Timer` trait.
Other than the `ErrorType` trait, none of the existing traits inherit from something else.
For `Alarm` it makes sense though, since it needs a time reference which `Timer` provides.

The alarm can only be awaited with one value at a time. This is to keep things simple. If required, people can write their own alarm queue on top if they need multiple alarms.

## Considerations

- The traits are not generically fallible and thus can't communicate specific errors.
  Similar to `DelayNs` these traits are meant to be used with internal hardware timers and so don't need more fallibility.
- Overflow is an error. If it were not, it could overflow and the user would get a low number returned which would be unexpected in most cases.
- All time values are `u64`. This is wasteful for performance, but allows the trait to be used much more widely. Previous versions of the proposal use a `u32`, but that was deemed to have the wrond tradeoff. Additionally, looking into the wider embedded ecosystem, we see timer being 64-bit more than 32-bit. Embassy-time uses 64-bit and that has been ok for most people and in Zephyr the 32-bit timers are somewhat deprecated in favor of the 64-bit ones.
We should follow suit.

## Why do we need this?

### `Timer`

There's currently no way to measure the time between two events. For example, people might want to write drivers that can measure the time between two gpio edge changes.

This trait would allow them to do that generically without relying on specific implementations.

### `Alarm`

For e.g. network stacks it's common to have long-running tasks doing multiple subtasks in a select.
An example would be a radio protocol implementation that needs to schedule broadcasts.
For trivial usecases you can go very far with `DelayNs`, but since `Future`s are anonymous, it gets tricky pretty fast since you need to keep the future on the stack.

## To discuss

- Normal bikeshedding
- More better docs
- What should the mutability for all functions be?
  - This is where `&mut self` helps implementations and `&self` helps sharability for the user which is a tension that needs a decision to be resolved
- What should the biggest and smallest (non-tick) resolution be? `DelayNs` goes down to nanoseconds, but only up to milliseconds.

## The case against `Alarm`

The `Alarm` trait is not 100% required when you have access to the `Timer` and `DelayNs`.

The way to emulate an alarm is:
```rust
let alarm_time_us = // Some value...
let current_time_us = timer.elapsed_micros().unwrap();

if alarm_time_us > current_time_us {
    delay.delay_us(alarm_time_us - current_time_us).await;
}
```

This is however clunky, possibly requires two timers instead of just one on simple implementations and is likely less accurate.

## Prior art

- Extended matrix discussion in the embedded room: https://libera.irclog.whitequark.org/rust-embedded/2024-05-22#1716404604-1716414859
- The 0.2 `CountDown` trait: https://docs.rs/embedded-hal/0.2.7/embedded_hal/timer/trait.CountDown.html
- TODO: Discussion/written reason about why `CountDown` was removed from the 1.0 release
- Unconf 2026 discussion: https://hackmd.io/@jnkr-ifx/S1Lplv3kGg
