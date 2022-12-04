mod dynamic_mapping;
mod fixed_mapping;
mod jumping_window;
mod mapping;

pub use dynamic_mapping::DynamicMapping;
pub use fixed_mapping::FixedMapping;
pub use jumping_window::JumpingWindow;

#[cfg(test)]
mod test {
    use std::{
        sync::Arc,
        thread,
        time::{Duration, SystemTime},
    };

    use crate::FixedMapping;

    fn benchmark_once(cooldown: Arc<FixedMapping<i32>>, id: i32, triggers: i32) {
        let start = SystemTime::now();
        for i in (triggers * id)..(triggers * (id + 1)) {
            cooldown.trigger(&i);
        }

        println!("Task took {:?}", start.elapsed());
    }

    #[test]
    fn benchmark() {
        const THREADS: i32 = 4;
        const TRIGGERS: i32 = 100_000 / THREADS;
        const CAPACITY: u64 = 10;
        const PERIOD: Duration = Duration::from_secs(1);

        let cooldown = FixedMapping::new(CAPACITY, PERIOD);
        let cooldown = Arc::new(cooldown);
        // FixedMapping::start(cooldown.clone(), None);

        let mut tasks = Vec::new();
        for id in 0..THREADS {
            let cld = cooldown.clone();
            tasks.push(thread::spawn(move || benchmark_once(cld, id, TRIGGERS)));
        }

        for t in tasks {
            t.join().unwrap();
        }
    }
}
