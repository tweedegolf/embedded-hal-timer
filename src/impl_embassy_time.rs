use crate::{Alarm, OverflowError, Timer};
use core::cell::Cell;
use critical_section::Mutex;

pub struct EmbassyTimeTimer(Mutex<Cell<u64>>);

impl EmbassyTimeTimer {
    pub fn new() -> Self {
        Self(Mutex::new(Cell::new(
            embassy_time::Instant::now().as_ticks(),
        )))
    }

    fn get_instant(&self) -> embassy_time::Instant {
        let ticks = critical_section::with(|cs| self.0.borrow(cs).get());
        embassy_time::Instant::from_ticks(ticks)
    }
}

impl Timer for EmbassyTimeTimer {
    fn start(&self) {
        let now = embassy_time::Instant::now();
        critical_section::with(|cs| self.0.borrow(cs).set(now.as_ticks()));
    }

    #[cfg(feature = "ticks-api")]
    fn tickrate(&self) -> u32 {
        embassy_time::TICK_HZ.try_into().unwrap()
    }

    #[cfg(feature = "ticks-api")]
    fn elapsed_ticks(&self) -> Result<u32, OverflowError> {
        u32::try_from(self.get_instant().elapsed().as_ticks()).map_err(|_| OverflowError)
    }

    fn elapsed_micros(&self) -> Result<u32, OverflowError> {
        u32::try_from(self.get_instant().elapsed().as_micros()).map_err(|_| OverflowError)
    }

    fn elapsed_millis(&self) -> Result<u32, OverflowError> {
        u32::try_from(self.get_instant().elapsed().as_millis()).map_err(|_| OverflowError)
    }

    fn elapsed_secs(&self) -> Result<u32, OverflowError> {
        u32::try_from(self.get_instant().elapsed().as_secs()).map_err(|_| OverflowError)
    }

    #[cfg(feature = "max-api")]
    fn max_micros(&self) -> u32 {
        embassy_time::Instant::MAX
            .as_micros()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    #[cfg(feature = "max-api")]
    fn max_millis(&self) -> u32 {
        embassy_time::Instant::MAX
            .as_millis()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    #[cfg(feature = "max-api")]
    fn max_secs(&self) -> u32 {
        embassy_time::Instant::MAX
            .as_secs()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    #[cfg(all(feature = "max-api", feature = "ticks-api"))]
    fn max_ticks(&self) -> u32 {
        u32::MAX
    }
}

impl Alarm for EmbassyTimeTimer {
    #[cfg(feature = "ticks-api")]
    async fn wait_until_ticks(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(
            self.get_instant() + embassy_time::Duration::from_ticks(value as u64),
        )
        .await;
        Ok(())
    }

    async fn wait_until_micros(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(
            self.get_instant() + embassy_time::Duration::from_micros(value as u64),
        )
        .await;
        Ok(())
    }

    async fn wait_until_millis(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(
            self.get_instant() + embassy_time::Duration::from_millis(value as u64),
        )
        .await;
        Ok(())
    }

    async fn wait_until_secs(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(
            self.get_instant() + embassy_time::Duration::from_secs(value as u64),
        )
        .await;
        Ok(())
    }
}
