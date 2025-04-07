use crate::{Alarm, OverflowError, Timer};

impl Timer for embassy_time::Instant {
    fn start(&mut self) {
        *self = Self::now();
    }

    #[cfg(feature = "ticks-api")]
    fn tickrate(&self) -> u32 {
        embassy_time::TICK_HZ.try_into().unwrap()
    }

    #[cfg(feature = "ticks-api")]
    fn elapsed_ticks(&mut self) -> Result<u32, OverflowError> {
        u32::try_from(self.elapsed().as_ticks()).map_err(|_| OverflowError)
    }

    fn elapsed_micros(&mut self) -> Result<u32, OverflowError> {
        u32::try_from(self.elapsed().as_micros()).map_err(|_| OverflowError)
    }

    fn elapsed_millis(&mut self) -> Result<u32, OverflowError> {
        u32::try_from(self.elapsed().as_millis()).map_err(|_| OverflowError)
    }

    fn elapsed_secs(&mut self) -> Result<u32, OverflowError> {
        u32::try_from(self.elapsed().as_secs()).map_err(|_| OverflowError)
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

impl Alarm for embassy_time::Instant {
    #[cfg(feature = "ticks-api")]
    async fn wait_until_ticks(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(*self + embassy_time::Duration::from_ticks(value as u64)).await;
        Ok(())
    }

    async fn wait_until_micros(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(*self + embassy_time::Duration::from_micros(value as u64)).await;
        Ok(())
    }

    async fn wait_until_millis(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(*self + embassy_time::Duration::from_millis(value as u64)).await;
        Ok(())
    }

    async fn wait_until_secs(&mut self, value: u32) -> Result<(), OverflowError> {
        embassy_time::Timer::at(*self + embassy_time::Duration::from_secs(value as u64)).await;
        Ok(())
    }
}
