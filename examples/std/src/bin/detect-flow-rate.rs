use embassy_executor::{Executor, Spawner};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant};
use embedded_hal_timer::Timer;
use static_cell::StaticCell;

static DETECT_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
const FLOW_PER_TICK: f32 = 0.45;

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    spawner.must_spawn(signaller());

    let mut flow_rate =
        FlowRateDriver::new(FLOW_PER_TICK, &DETECT_SIGNAL, embassy_time::Instant::now());

    println!("Starting flow detection");
    let mut total_flow = 0.0;
    loop {
        let current_flow = flow_rate.wait().await;
        total_flow += current_flow;
        println!("Detected flow: {current_flow}, total: {total_flow}\n",);
    }
}

#[embassy_executor::task]
async fn signaller() {
    let mut instant = Instant::now();
    let mut total_flow = 0.0;

    loop {
        let time_to_next_tick = rand::random_range(500_000..1_000_000);
        instant += Duration::from_micros(time_to_next_tick);
        embassy_time::Timer::at(instant).await;
        DETECT_SIGNAL.signal(());
        let current_flow = FLOW_PER_TICK / ((time_to_next_tick as f32) / 1_000_000.0);
        total_flow += current_flow;

        println!("Flow should be: {current_flow}, total: {total_flow}",);
    }
}

struct FlowRateDriver<T: Timer> {
    flow_rate_per_tick: f32,
    detect: &'static Signal<CriticalSectionRawMutex, ()>,
    timer: T,
}

impl<T: Timer> FlowRateDriver<T> {
    fn new(
        flow_rate_per_tick: f32,
        detect: &'static Signal<CriticalSectionRawMutex, ()>,
        mut timer: T,
    ) -> Self {
        timer.start();

        Self {
            flow_rate_per_tick,
            detect,
            timer,
        }
    }

    async fn wait(&mut self) -> f32 {
        self.detect.wait().await;
        let elapsed_micros = self.timer.elapsed_micros().unwrap();
        self.timer.start(); // Slight difference between getting elapsed time and restarting the timer...

        let elapsed_seconds = (elapsed_micros as f32) / 1_000_000.0;

        self.flow_rate_per_tick / elapsed_seconds
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
