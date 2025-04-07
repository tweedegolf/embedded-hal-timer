# Embedded-hal `Timer` and `Alarm`

## Existing time constructs in `embedde-hal(-async)`

Currently we can do a delay using the sync or async `DelayNs` trait. This is good for e.g. initializing chips that need a little delay between two commands/writes.

The delay is modeled with a simple trait without fluff. You can delay for a number of nanos, micros or millis. They are all a u32 and the contract is that the delay will be *at least* as long as the specified amount of time.
This means there's also a tradeoff being done where you can't specify a long delay (enough) with an exact amount of nanoseconds.

## Other time needs in platform-independent code

### Time between two events

With an async delay you can make a timeout, however if you simply wish to know the duration between two points you're out of luck.

Consider someone wants to write a generic driver for a pulse meter. An example of this is a water meter that pulses an LED for every liter that has flowed through it. Today this could not be written using embedded-hal traits.

An async example to illustrate this:

```rust
async fn detect_flow_rate(input: &mut impl Wait) -> f32 {
    input.wait_for_high().await;
    let first_pulse_time = // ...?
    input.wait_for_low().await;

    input.wait_for_high().await;
    let second_pulse_time = // ...?
    input.wait_for_low().await;

    let secs_passed = second_pulse_time - first_pulse_time;
    1 / secs_passed * LITERS_PER_PULSE
}
```

There is no way to get the time or to get the duration between two events.
The example is limited and has its problems but that's besides the point of illustration.

### A delay that has been started before (or periodics)

_This section applies only or mostly to async delay._

The `DelayNs` trait is nice to do timeouts with and wait until a time has come.
However, the delay always starts when you call one of its functions and when you cancel an async delay you don't get the time of how long it's been running.

This can become a problem in more complex programs.
Consider this example where we want to do something periodically:

```rust
async fn wait_for_event(&mut self) -> Event {
    loop {
        let either = select(self.delay.delay_ms(100), self.input.wait_for_any_edge()).await;
        
        match either {
            First(_) => self.do_some_bookkeeping(),
            Second(_) => return self.process_event(),
        }
    }
}

// In user code:
loop {
    process_driver_event(driver.wait_for_event().await);
}
```

In this case we do the bookkeeping, but only when the input doesn't trigger for more than 100ms. When the input is triggered, it resets the delay. This can be prevent, but only with a lot of nasty code since the delay is an unnamed future that can't easily be stored in the driver struct.

## Solution: `Timer` trait

These issues can be solved with a new trait that allows to get the duration between 'now' and a previous starting point.
There have been discussions in the past on what that should look like.

People didn't like:
- A freerunning clock
- Too much complexity

People said it should be:
- Easy to implement on hardware
- Meant for 'short' durations

The suggested trait then still satisfies all demands:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct OverflowError;

pub trait Timer {
    /// Start or restart the timer at 0.
    fn start(&mut self);
    /// Return the number of elapsed microseconds, rounded down.
    fn elapsed_micros(&mut self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed milliseconds, rounded down.
    fn elapsed_millis(&mut self) -> Result<u32, OverflowError>;
    /// Return the number of elapsed seconds, rounded down.
    fn elapsed_secs(&mut self) -> Result<u32, OverflowError>;
}
```

## Solution: `Alarm` trait

```rust
pub trait Alarm: Timer {
    /// Set the alarm to a number of microseconds after the timer start, rounded up.
    fn set_alarm_micros(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Set the alarm to a number of milliseconds after the timer start, rounded up.
    fn set_alarm_millis(&mut self, value: u32) -> Result<(), OverflowError>;
    /// Set the alarm to a number of seconds after the timer start, rounded up.
    fn set_alarm_secs(&mut self, value: u32) -> Result<(), OverflowError>;

    /// Wait until the timer reaches the alarm. If the alarm is already reached, the function exits immediately.
    async fn wait(&mut self);
}
```

## Updated examples:

```rust
async fn detect_flow_rate(input: &mut impl Wait, timer: &mut impl Timer) -> f32 {
    input.wait_for_high().await;
    timer.start();
    input.wait_for_low().await;

    input.wait_for_high().await;
    let secs_passed = timer.elapsed_micros().unwrap() as f32 / 1_000_000.0;
    input.wait_for_low().await;

    1 / secs_passed * LITERS_PER_PULSE
}
```

```rust
async fn wait_for_event(&mut self) -> Event {
    self.timer.start();
    self.timer.set_alarm_millis(100).unwrap();

    loop {
        let either = select(self.timer.wait(), self.input.wait_for_any_edge()).await;
        
        match either {
            First(_) => {
                self.timer.start(); // Restart the timer
                // TODO:"Should we need to set the alarm again or would it keep the old one?
                self.do_some_bookkeeping();
            }
            Second(_) => return self.process_event(),
        }
    }
}

// In user code:
loop {
    process_driver_event(driver.wait_for_event().await);
}
```

## Considerations

The `Timer` trait is very simple. It can overflow, is not async and not fallible.
The intention of the trait is very much to be run with MCU internal timers.
For the overflow, it should be up to the driver to document how long it expects timers to be able to run and then it's up to the user to choose a good implementation.

## Open questions

### Ticks
Should the `Timer` and `Alarm` traits have functions for using the raw ticks?
It could have these extra functions:
- `fn elapsed_ticks(&mut self) -> u32;`
- `fn hertz(&self) -> u32;`
These functions provide more information about the timer that could be useful for extra precision or extra performance (e.g. when you only need to compare ticks for which one is longer and don't care about the millisecond values)

### Overflow values
If we want drivers to be able to reject a timer that can't run long enough, that information should be exposed.
There are two ways to do that:
- An associated const
  - Pro: The driver could put bounds or do a static assert on the value which rejects it at compile time
  - Con: The max needs to be known at compile time. Max ticks is doable, but the tickrate is likely not known, so e.g. max millis can't be known
- A function (getter)

### Alarm

Having the alarm trait would be nice, but not required since you can get close with a `Timer` and a `DelayNs`. However, it would be a lot more awkward to use and might on some platforms require two timers instead of just one.

The API right now also has different functions for setting the alarm and waiting on it. Instead these functions could be combined into a `async fn wait_until_millis(value: u32) -> Result<(), OverflowError>`. We'd have to try implementing things for a real timer to see which is nicer to use.