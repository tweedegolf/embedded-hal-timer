use crate::OverflowError;
use embassy_stm32::{
    pac::timer::vals::Urs,
    timer::{CoreInstance, low_level::Timer},
};

impl<'a, T: CoreInstance> crate::Timer for Timer<'a, T> {
    fn start(&self) {
        critical_section::with(|_| {
            self.regs_core().cr1().modify(|reg| {
                reg.set_urs(Urs::COUNTER_ONLY);
                reg.set_opm(true);
                reg.set_udis(false);
            });

            self.regs_core().arr().write(|reg| reg.set_arr(u16::MAX));
            // Generate an Update Request
            self.regs_core().egr().write(|r| r.set_ug(true));
            self.regs_core().sr().modify(|reg| reg.set_uif(false));

            Timer::reset(self);
            Timer::start(self);
        });
    }

    fn tickrate(&self) -> u32 {
        self.get_clock_frequency().0 / (self.regs_core().psc().read() + 1) as u32
    }

    fn elapsed_ticks(&self) -> Result<u32, OverflowError> {
        if self.regs_core().sr().read().uif() {
            return Err(OverflowError);
        }

        Ok(self.regs_core().cnt().read().cnt() as u32)
    }

    fn elapsed_micros(&self) -> Result<u32, OverflowError> {
        Ok(((self.elapsed_ticks()? as u64 * 1_000_000u64) / self.tickrate() as u64) as u32)
    }

    fn elapsed_millis(&self) -> Result<u32, OverflowError> {
        Ok((self.elapsed_ticks()? * 1000) / self.tickrate())
    }

    fn elapsed_secs(&self) -> Result<u32, OverflowError> {
        Ok(self.elapsed_ticks()? / self.tickrate())
    }

    fn max_micros(&self) -> u32 {
        ((self.max_ticks() as u64 * 1_000_000u64) / self.tickrate() as u64) as u32
    }

    fn max_millis(&self) -> u32 {
        (self.max_ticks() * 1000) / self.tickrate()
    }

    fn max_secs(&self) -> u32 {
        self.max_ticks() / self.tickrate()
    }

    fn max_ticks(&self) -> u32 {
        u16::MAX as u32
    }
}

// No alarm impl because that's hard to do with just the public embassy-stm32 api
// But with a timer that has a compare channel it could be easily implemented
