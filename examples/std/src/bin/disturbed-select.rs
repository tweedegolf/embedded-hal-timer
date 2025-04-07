use embassy_executor::{Executor, Spawner};
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant};
use embedded_hal_timer::Alarm;
use static_cell::StaticCell;

static DISTURBER: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    spawner.must_spawn(signaller());

    run_disturber_with_regular(&DISTURBER, Instant::now()).await;
}

#[embassy_executor::task]
async fn signaller() {
    loop {
        embassy_time::Timer::after(Duration::from_micros(rand::random_range(
            100_000..1_000_000,
        )))
        .await;
        DISTURBER.signal(());
    }
}

async fn run_disturber_with_regular(
    disturber: &Signal<CriticalSectionRawMutex, ()>,
    mut alarm: impl Alarm,
) {
    loop {
        match select(disturber.wait(), alarm.wait_until_secs(1)).await {
            Either::First(_) => {
                println!("** Disturber @ {} **", Instant::now().as_millis());
            }
            Either::Second(r) => {
                r.unwrap();
                alarm.start();
                println!("Regular @ {}", Instant::now().as_millis());
            }
        }
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
