#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{exti::ExtiInput, time::khz};
use embedded_hal_async::digital::Wait;
use embedded_hal_timer::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let button = ExtiInput::new(p.PC13, p.EXTI13, embassy_stm32::gpio::Pull::Down);
    let mut timer = embassy_stm32::timer::low_level::Timer::new(p.TIM17);
    timer.set_tick_freq(khz(100));

    info!(
        "Press the button!\nBut not for longer than {=u32} secs, {=u32} millis or {=u32} micros...\nThe tickrate is: {=u32}",
        timer.max_secs(),
        timer.max_millis(),
        timer.max_micros(),
        timer.tickrate(),
    );

    measure_button(button, timer).await;
}

async fn measure_button(mut button: impl Wait, mut timer: impl Timer) -> ! {
    loop {
        button.wait_for_low().await.unwrap();
        timer.start();
        button.wait_for_high().await.unwrap();
        let elapsed = timer.elapsed_micros();

        match elapsed {
            Ok(val) => defmt::info!("Button was high for {=u32} us", val),
            Err(_) => defmt::info!("Button was high for too long"),
        }
    }
}
