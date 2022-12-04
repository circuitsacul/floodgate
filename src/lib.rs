mod dynamic_mapping;
mod fixed_mapping;
mod jumping_window;
mod mapping;

pub use dynamic_mapping::DynamicMapping;
pub use fixed_mapping::FixedMapping;
pub use jumping_window::JumpingWindow;

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    use crate::FixedMapping;

    #[test]
    fn benchmark() {
        let mapping = Arc::new(FixedMapping::<u64>::new(1, Duration::from_secs(2)));
        FixedMapping::start(mapping.clone(), None);

        let start = Instant::now();
        for i in 0..10_000_000 {
            mapping.trigger(&i);
        }
        println!("Elapsed: {:?}", start.elapsed());
    }
}
