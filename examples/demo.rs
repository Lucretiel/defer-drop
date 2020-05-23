use defer_drop::DeferDrop;
use std::iter::repeat_with;
use std::time::{Duration, Instant};

fn timer(f: impl FnOnce()) -> Duration {
    let start = Instant::now();
    f();
    start.elapsed()
}

fn main() {
    println!("Allocating a ridiculously large vector");
    let vec1: Vec<Vec<String>> = repeat_with(|| {
        repeat_with(|| "Hello, World".to_string())
            .take(1000)
            .collect()
    })
    .take(1000)
    .collect();

    println!("Duplicating that vector");
    let vec2 = vec1.clone();
    let defer_vec1 = DeferDrop::new(vec1);

    println!("Dropping the vectors");

    let vec1_timer = timer(move || drop(defer_vec1));
    let vec2_timer = timer(move || drop(vec2));

    println!("Duration of deferred drop: {:?}", vec1_timer);
    println!("Duration of foreground drop: {:?}", vec2_timer);
}
