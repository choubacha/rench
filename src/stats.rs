use std::time::Duration;
use reqwest::{header, StatusCode, Response};
use std::fmt;
use chart::Chart;
use std::cmp;
use content_length::ContentLength;

trait ToMilliseconds {
    fn to_ms(&self) -> f64;
}

impl ToMilliseconds for Duration {
    fn to_ms(&self) -> f64 {
        (self.as_secs() as f64 * 1_000f64) + (self.subsec_nanos() as f64 / 1_000_000f64)
    }
}

#[cfg(test)]
mod millisecond_tests {
    use super::*;

    #[test]
    fn exchange_duration_to_ms() {
        assert_eq!(Duration::new(1, 500000).to_ms(), 1000.5f64);
    }
}



#[derive(Debug)]
pub struct Fact {
    status: StatusCode,
    duration: Duration,
    content_length: ContentLength,
}

impl Fact {
    pub fn record(resp: Response, duration: Duration) -> Fact {
        let content_length = resp
                .headers()
                .get::<header::ContentLength>()
                .map(|len| **len)
                .unwrap_or(0);

        Fact {
            duration,
            status: resp.status(),
            content_length: ContentLength::new(content_length),
        }
    }
}

struct DurationStats {
    sorted: Vec<Duration>
}

impl DurationStats {
    fn from_facts(facts: &[Fact]) -> DurationStats {
        let mut sorted: Vec<Duration> = facts.iter().map(|f| f.duration.clone()).collect();
        sorted.sort();
        Self { sorted }
    }

    fn max(&self) -> Option<Duration> {
        self.sorted.last().map(|d| *d)
    }

    fn min(&self) -> Option<Duration> {
        self.sorted.first().map(|d| *d)
    }

    fn median(&self) -> Duration {
        let mid = self.sorted.len() / 2;
        if self.sorted.len() % 2 == 0 {
            // even
            (self.sorted[mid - 1] + self.sorted[mid]) / 2
        } else {
            // odd
            self.sorted[mid]
        }
    }

    fn average(&self) -> Duration {
        self.total() / self.sorted.len() as u32
    }

    fn latency_histogram(&self) -> Vec<u32> {
        let mut latency_histogram = vec![0; 50];

        if let Some(max) = self.max() {
            let bin_size = max.to_ms() / 50.;

            for duration in &self.sorted {
                let index = (duration.to_ms() / bin_size) as usize;
                latency_histogram[cmp::min(index, 49)] += 1;
            }
        }
        latency_histogram
    }

    fn percentiles(&self) -> Vec<Duration> {
        (0..50)
            .map(|n| {
                let mut index = ((n as f64 / 50.0) * self.sorted.len() as f64) as usize;
                index = cmp::max(index, 0);
                index = cmp::min(index, self.sorted.len() - 1);
                self.sorted[index]
            })
            .collect()
    }

    fn total(&self) -> Duration {
        self.sorted.iter().sum()
    }
}

#[derive(Debug)]
pub struct Summary {
    average: Duration,
    median: Duration,
    max: Duration,
    min: Duration,
    count: u32,
    content_length: ContentLength,
    percentiles: Vec<Duration>,
    latency_histogram: Vec<u32>,
}

impl Summary {
    pub fn from_facts(facts: &[Fact]) -> Summary {
        if facts.len() == 0 {
            return Summary::zero();
        }
        let content_length  = Self::total_content_length(&facts);
        let count           = facts.len() as u32;

        Summary {
            count,
            content_length,
            ..Summary::from_durations(DurationStats::from_facts(&facts))
        }
    }

    fn from_durations(stats: DurationStats) -> Summary {
        let average             = stats.average();
        let median              = stats.median();
        let min                 = stats.min().expect("Returned early if empty");
        let max                 = stats.max().expect("Returned early if empty");
        let latency_histogram   = stats.latency_histogram();
        let percentiles         = stats.percentiles();

        Summary {
            average,
            median,
            min,
            max,
            percentiles,
            latency_histogram,
            ..Summary::zero()
        }
    }

    fn zero() -> Summary {
        Summary {
            average: Duration::new(0, 0),
            median: Duration::new(0, 0),
            max: Duration::new(0, 0),
            min: Duration::new(0, 0),
            count: 0,
            content_length: ContentLength::zero(),
            percentiles: vec![Duration::new(0, 0); 100],
            latency_histogram: vec![0; 0],
        }
    }

    fn total_content_length(facts: &[Fact]) -> ContentLength {
        facts.iter().fold(ContentLength::zero(), |len, fact| len + &fact.content_length)
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Summary")?;
        writeln!(f, "  Average:   {} ms", self.average.to_ms())?;
        writeln!(f, "  Median:    {} ms", self.median.to_ms())?;
        writeln!(f, "  Longest:   {} ms", self.max.to_ms())?;
        writeln!(f, "  Shortest:  {} ms", self.min.to_ms())?;
        writeln!(f, "  Requests:  {}", self.count)?;
        writeln!(f, "  Data:      {}", self.content_length)?;
        writeln!(f, "")?;
        writeln!(f, "Latency Percentiles (2% of requests per bar):")?;
        let percentiles: Vec<f64> = self.percentiles.iter().map(|d| d.to_ms()).collect();
        writeln!(f, "{}", Chart::new().make(&percentiles))?;
        writeln!(f, "")?;
        writeln!(f, "Latency Histogram (each bar is 2% of max latency)")?;
        writeln!(f, "{}", Chart::new().make(&self.latency_histogram))?;
        Ok(())
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    fn ok_zero_length_fact(duration: Duration) -> Fact {
        Fact {
            status: StatusCode::Ok,
            duration: duration,
            content_length: ContentLength::zero(),
        }
    }

    fn ok_instant_fact(content_length: ContentLength) -> Fact {
        Fact {
            status: StatusCode::Ok,
            duration: Duration::new(0, 0),
            content_length,
        }
    }

    #[test]
    fn summarizes_to_zero_if_empty() {
        let summary = Summary::from_facts(&Vec::new());
        assert_eq!(summary.average, Duration::new(0, 0));
        assert_eq!(summary.median, Duration::new(0, 0));
        assert_eq!(summary.count, 0);
    }

    #[test]
    fn averages_the_durations() {
        let facts = [
            ok_zero_length_fact(Duration::new(2, 0)),
            ok_zero_length_fact(Duration::new(1, 0)),
            ok_zero_length_fact(Duration::new(4, 0)),
            ok_zero_length_fact(Duration::new(3, 0)),
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.average, Duration::new(2, 500000000));
    }

    #[test]
    fn counts_the_facts() {
        let facts = [
            ok_zero_length_fact(Duration::new(2, 0)),
            ok_zero_length_fact(Duration::new(3, 0)),
            ok_zero_length_fact(Duration::new(1, 0)),
            ok_zero_length_fact(Duration::new(4, 0)),
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.count, 4);
    }

    #[test]
    fn calculates_percentiles_from_an_even_number_of_facts() {
        let facts = [
            ok_zero_length_fact(Duration::new(2, 0)),
            ok_zero_length_fact(Duration::new(3, 0)),
            ok_zero_length_fact(Duration::new(1, 0)),
            ok_zero_length_fact(Duration::new(100, 0)),
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.median, Duration::new(2, 500000000));
        assert_eq!(summary.max, Duration::new(100, 0));
        assert_eq!(summary.min, Duration::new(1, 0));
    }

    #[test]
    fn calculates_percentiles_from_an_odd_number_of_facts() {
        let facts = [
            ok_zero_length_fact(Duration::new(2, 0)),
            ok_zero_length_fact(Duration::new(1, 0)),
            ok_zero_length_fact(Duration::new(100, 0)),
        ];
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.median, Duration::new(2, 0));
        assert_eq!(summary.max, Duration::new(100, 0));
        assert_eq!(summary.min, Duration::new(1, 0));
    }

    #[test]
    fn counts_the_histogram_of_latencies() {
        let facts: Vec<Fact> = (0..500)
            .map(|n| ok_zero_length_fact(Duration::new(n, 0)))
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.latency_histogram.len(), 50);
        assert_eq!(summary.latency_histogram.first(), Some(&10));
        assert_eq!(summary.latency_histogram.last(), Some(&10));
        assert_eq!(summary.latency_histogram[25], 10);
    }

    #[test]
    fn calculates_all_the_percentiles_when_n_less_than_100() {
        let facts: Vec<Fact> = (0..50)
            .map(|n| ok_zero_length_fact(Duration::new(n, 0)))
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.percentiles.len(), 50);
        assert_eq!(summary.percentiles.first(), Some(&Duration::new(0, 0)));
        assert_eq!(summary.percentiles.last(), Some(&Duration::new(49, 0)));
        assert_eq!(summary.percentiles[25], Duration::new(25, 0));
    }

    #[test]
    fn calculates_all_the_percentiles_when_n_greater_than_100() {
        let facts: Vec<Fact> = (0..500)
            .map(|n| ok_zero_length_fact(Duration::new(n, 0)))
            .collect();
        let summary = Summary::from_facts(&facts);

        assert_eq!(summary.percentiles.len(), 50);
        assert_eq!(summary.percentiles.first(), Some(&Duration::new(0, 0)));
        assert_eq!(summary.percentiles.last(), Some(&Duration::new(490, 0)));
        assert_eq!(summary.percentiles[25], Duration::new(250, 0));
    }

    #[test]
    fn sums_up_the_content_lengths() {
        let facts: Vec<Fact> = (0..500)
            .map(|_| ok_instant_fact(ContentLength::new(1)))
            .collect();
        let summary = Summary::from_facts(&facts);
        assert_eq!(summary.content_length.bytes(), 500);
    }
}
