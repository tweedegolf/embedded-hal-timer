# Embedded-hal `Timer` and `Alarm`

Time handling in `embedded-hal` is limited to simple delays.
While a full all-encompasing time system like `embassy-time` is likely out of scope, it would be helpful if some of the gaps are plugged.

The main proposal is a `Timer` trait for measuring the time between two or more points, but only for short durations.
Using `Timer` as a monotonic timer for the time since startup is out of scope.
Additionally an `Alarm` trait is proposed that makes the `Timer` easier to use in some async contexts (instead of having to juggle both a `Timer` and a `DelayNs`).

If this RFC were accepted as is, there'd be three traits that have something to do with time:
- `DelayNs` (sync and async, already exists): Wait for a specified amount of time
- `Timer`: Start a running time at 0, and query how long it has been running
- `Alarm`: Wait until a `Timer` value has been reached

This proposal has two extensions that can be accepted or not depending on whether the tradeoff is worth it.
Here they are represented using crate features, but when accepted they should be fully part of the trait without cfg gates.

Find the traits and some implementations here: https://github.com/tweedegolf/embedded-hal-timer

## `Timer`

```rust
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
    fn elapsed_ticks(&mut self) -> Result<u32, OverflowError>;

    /// Return the number of elapsed microseconds, rounded down.
    fn elapsed_micros(&mut self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed milliseconds, rounded down.
    fn elapsed_millis(&mut self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed seconds, rounded down.
    fn elapsed_secs(&mut self) -> Result<u32, OverflowError>;

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
```

In its simplest form you can start (and restart) the timer and get its elapsed time since the start.
The returned elapsed time is rounded down, so the contract is that *at least* this amount of time has passed.

The `ticks` extension adds the ability to inspect the raw tick count and the tickrate.
In some situations this allows the user to do more accurate math manually.
Any serious/embedded implementation of `Timer` will already carry this information internally, so it's cheap/free to expose it to the user.

The `max` extension add the ability to expect the value at which the implementation will overflow.
This can help a driver reject an implementation if its max time is too short.

## `Alarm`

```rust
/// An alarm that can be used to wait for a time to come.
#[allow(async_fn_in_trait)]
pub trait Alarm: Timer {
    #[cfg(feature = "ticks-api")]
    /// Wait until the timer reaches the alarm specified in ticks. If the alarm is already reached, the function exits immediately.
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_ticks(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in microseconds rounded up. If the alarm is already reached, the function exits immediately.
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_micros(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in milliseconds rounded up. If the alarm is already reached, the function exits immediately.
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_millis(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Wait until the timer reaches the alarm specified in seconds rounded up. If the alarm is already reached, the function exits immediately.
    /// The function returns an overflow error if the alarm value is higher than is supported by the implementation.
    async fn wait_until_secs(&mut self, value: u32) -> Result<(), OverflowError>;
}
```

Probably the most contentious part of the `Alarm` trait is that it inherits from the `Timer` trait.
Other than the `ErrorType` trait, none of the existing traits inherit from something else.
For `Alarm` it makes sense though, since it needs a time reference which `Timer` already provides.

## Considerations

- The traits are not generally fallible. Similar to `DelayNs` these traits are meant to be used with internal hardware timers.
- Overflow is an error. If it were not, it could overflow and the user would get a low number returned which would be unexpected in most cases.
- All time values are `u32` since this is usable in most usecases and most people are using 32-bit hardware.

## Why do we need this?

### `Timer`

Measuring the time is useful in a lot of cases. The example here will be a driver for a flow meter.
These devices often give a pulse every X amount of liquid or gas that has passed the meter.
Currently this can't be built with only `embedded-hal` traits.

With the timer one could build (with an example of how the `ticks` extension could be useful):

```rust
async fn print_flow_rate(mut input: impl Wait, mut timer: impl Timer) {
    input.wait_for_high().await;
    timer.start();
    input.wait_for_low().await;

    loop {
        input.wait_for_high().await;
        #[cfg(feature = "ticks-api")]
        let secs_passed = timer.elapsed_ticks().unwrap() as f32 / timer.tickrate() as f32;
        #[cfg(not(feature = "ticks-api"))]
        let secs_passed = timer.elapsed_micros().unwrap() as f32 / 1_000_000.0;

        timer.start(); // Restart the timer
        input.wait_for_low().await;
    
        let flow_rate = 1.0 / secs_passed * LITERS_PER_PULSE;

        info!("Current flow: {}", flow_rate);
    }
}
```

A real implementation would build support for the situation where you'd have extended times of 0 flow,
which this example does not handle gracefully.

### `Alarm`

It's a common pattern to have a long-running task doing multiple things in a select.
An example would be a radio protocol implementation that needs to schedule broadcasts.
For trivial example you can go very far with `DelayNs`, but since `Future`s are anonymous, it gets tricky pretty fast.

A good usecase for an alarm would be:

```rust
async fn wait_for_event(&mut self) -> Event {
    let schedule_timer: impl Alarm = &mut self.schedule_timer;
    let interval = self.get_scheduled_interval();
    let message_bus = &mut self.message_bus;

    match select(schedule_timer.wait_until_millis(interval), message_bus.recv()).await {
        First(_) => {
            schedule_timer.start(); // Restart the schedule timer
            Event::ScheduledBroadcast
        },
        Second(message) => {
            Event::Message(message)
        }
    }
}
```

## To discuss

- Normal bikeshedding
- More better docs
- If time is measured in loops, such as in the `print_flow_rate` example, then there is some time difference between reading the elapsed time and resetting it back to 0
  - Maybe this is not acceptable and points to a possibility of a better API
  - Maybe this can only really be solved by long-running timers and we should come up with a solution for that
    - This could be switching to `u64` and not dealing with overflows in the trait design like `embassy-time`.
    - This would make the implementation much more involved.
- The alarm example would be better off with some sort of `Ticker` abstraction à la `embassy_time::Ticker`.
  - Maybe this suggests the `Alarm` trait is less useful than is presented in this proposal.
- Maybe the 'max' `Timer` values should be associated consts?
  - Will they always be known at compile time?
- Maybe the `Alarm` should keep track of the alarm value. The API would then roughly become:
  ```rust
  pub trait Alarm: Timer {
      /// Set the alarm to a number of microseconds after the timer start, rounded up.
      fn set_alarm_micros(&mut self, value: u32) -> Result<(), OverflowError>;
      // ...

      /// Wait until the timer reaches the alarm.
      /// If the alarm is already reached, the function exits immediately.
      async fn wait(&mut self);
  }
  ```
  - This would allow priming the alarm ahead of time which *could* make things easier for the user, especially if the alarm value is kept after restart. This would be at the cost of potentially higher implementation complexity.
- Which extensions do we want to include in the final result?
- Is overflow the only error we want to give?
  - What should happen when the timer hasn't started yet?
  - Implementation would likely get simpler if the overflow error (but perhaps with a different name) would be allowed to be returned when the timer hasn't started yet

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
