use std::time::{Duration, Instant};

/// Executes a closure once and returns how long that closure took
/// as a duration.
pub fn time_it<F, U>(f: F) -> (U, Duration)
where
    F: FnOnce() -> U,
{
    let start = Instant::now();
    (f(), start.elapsed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_duration_and_response() {
        let (u, d) = time_it(|| 123);
        assert_eq!(u, 123);
        assert!(d > Duration::new(0, 0));
    }
}
