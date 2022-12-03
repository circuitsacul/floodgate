mod fixed_mapping;
mod jumping_window;
mod mapping;

pub use fixed_mapping::FixedMapping;
pub use jumping_window::JumpingWindow;

#[cfg(feature="tokio")]
#[cfg(test)]
mod test {
    use std::{time::{Duration, SystemTime}, sync::Arc};

    use tokio::sync::RwLock;

    use crate::FixedMapping;

    async fn benchmark_once(cooldown: Arc<RwLock<FixedMapping<i32>>>) {
        const TRIGGERS: i32 = 1_000_000;

        let start = SystemTime::now();
        for i in 0..TRIGGERS {
            cooldown.read().await.trigger(&(i % 100_000));
        }

        println!("Task took {:?}", start.elapsed());
    }

    #[tokio::test]
    async fn benchmark() {
        const CAPACITY: u64 = 10;
        const PERIOD: Duration = Duration::from_secs(1);

        let cooldown = FixedMapping::new(CAPACITY, PERIOD);
        let cooldown = Arc::new(RwLock::new(cooldown));
        FixedMapping::start(cooldown.clone());

        let mut tasks = Vec::new();
        for _ in 0..10 {
            tasks.push(tokio::spawn(benchmark_once(cooldown.clone())));
        }

        for t in tasks {
            t.await.unwrap();
        }
    }
}
