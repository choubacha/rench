use std::time::{Duration, Instant};

pub fn time_it<F, U>(f: F) -> (U, Duration)
where
    F: FnOnce() -> U,
{
    let start = Instant::now();
    (f(), start.elapsed())
}
