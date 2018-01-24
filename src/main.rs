extern crate rayon;
extern crate clap;
extern crate reqwest;
use clap::{Arg, App};
use reqwest::{StatusCode, Request, Method, Client};
use std::time::{Instant, Duration};

#[derive(Debug)]
struct Fact {
    status: StatusCode,
    duration: Duration,
    content_length: usize,
}

#[derive(Debug)]
struct Summary {
    average: Duration,
    median: Duration,
    count: u32,
}

impl Summary {
    fn zero() -> Summary {
        Summary {
            average: Duration::new(0, 0),
            median: Duration::new(0, 0),
            count: 0
        }
    }
}

impl Summary {
    fn from_facts(facts: &[Fact]) -> Summary {
        if facts.len() == 0 {
            return Summary::zero()
        }
        let count = facts.len() as u32;
        let sum: Duration = facts.iter().map(|f| f.duration).sum();
        let average =  sum / count;
        let mut sorted: Vec<Duration> = facts.iter().map(|f| f.duration.clone()).collect();
        sorted.sort();

        let mid = facts.len() / 2;
        let median = if facts.len() % 2 == 0 { // even
                         (facts[mid - 1].duration + facts[mid].duration) / 2
                     } else { // odd
                         facts[mid].duration
                     };
        Summary { average, median, count }
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
        Fact { status: StatusCode::Ok, duration: Duration::new(1, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(2, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(3, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(4, 0), content_length: 0 },
    ];
    let summary = Summary::from_facts(&facts);
    assert_eq!(summary.average, Duration::new(2, 500000000));
}

#[test]
fn counts_the_facts() {
    let facts = [
        Fact { status: StatusCode::Ok, duration: Duration::new(1, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(2, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(3, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(4, 0), content_length: 0 },
    ];
    let summary = Summary::from_facts(&facts);
    assert_eq!(summary.count, 4);
}

#[test]
fn calculates_the_median_from_an_even_number_of_facts() {
    let facts = [
        Fact { status: StatusCode::Ok, duration: Duration::new(1, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(2, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(3, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(100, 0), content_length: 0 },
    ];
    let summary = Summary::from_facts(&facts);
    assert_eq!(summary.median, Duration::new(2, 500000000));
}

#[test]
fn calculates_the_median_from_an_odd_number_of_facts() {
    let facts = [
        Fact { status: StatusCode::Ok, duration: Duration::new(1, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(2, 0), content_length: 0 },
        Fact { status: StatusCode::Ok, duration: Duration::new(100, 0), content_length: 0 },
    ];
    let summary = Summary::from_facts(&facts);
    assert_eq!(summary.median, Duration::new(2, 0));
}

fn make_request(url: &str) {
    let client = Client::new();

    // Warm up
    let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
    let _ = client.execute(request).expect("Failure to even connect is no good");

    let facts: Vec<Fact> = (0..1000).map(|_| {
        let mut request = client.get(url);
        let start = Instant::now();
        let resp = request.send().expect("Failure to even connect is no good");
        let duration = start.elapsed();
        let fact = Fact { duration, status: resp.status(), content_length: 0, };
        println!("{:?}", fact);
        fact
    }).collect();
    println!("{:?}", Summary::from_facts(&facts));
}

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(Arg::with_name("URL").required(true))
        .get_matches();
    let url = matches.value_of("URL").expect("URL is required");
    make_request(url);
}
