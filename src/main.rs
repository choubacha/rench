extern crate rayon;
extern crate clap;
extern crate reqwest;
use clap::{Arg, App};
use reqwest::{Request, Method, Client};
use std::time::{Instant, Duration};
use std::thread;

mod stats;
use stats::{Fact, Summary};


fn make_requests(url: &str, number_of_requests: u32) -> Vec<Fact> {
    let client = Client::new();

    // Warm up
    let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
    let _ = client.execute(request).expect("Failure to warm connection");

    (0..number_of_requests)
        .map(|_| {
            let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
            let (resp, duration) = time_it(|| {
                let mut resp = client.execute(request).expect(
                    "Failure to even connect is no good",
                );
                let _ = resp.text().expect("Read the body");
                resp
            });
            Fact::record(resp, duration)
        })
        .collect()
}

fn time_it<F, U>(f: F) -> (U, Duration)
    where F: FnOnce() -> U
{
    let start = Instant::now();
    (f(), start.elapsed())
}

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(Arg::with_name("URL").required(true))
        .arg(Arg::with_name("concurrency").short("c").takes_value(true))
        .arg(Arg::with_name("requests").short("n").takes_value(true))
        .get_matches();

    let url = matches
        .value_of("URL")
        .expect("URL is required")
        .to_string();

    let threads = matches
        .value_of("concurrency")
        .unwrap_or("1")
        .parse::<u32>()
        .expect("Expected valid number for threads");

    let requests = matches
        .value_of("requests")
        .unwrap_or("1000")
        .parse::<u32>()
        .expect("Expected valid number for number of requests");

    let handles: Vec<thread::JoinHandle<Vec<Fact>>> = (0..threads)
        .map(|_| {
            let param = url.clone();
            thread::spawn(move || make_requests(&param, requests / threads))
        })
        .collect();
    let (facts, duration): (Vec<Vec<Fact>>, Duration) = time_it(|| {
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let seconds = duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1_000_000_000f64);
    println!("Took {} seconds", seconds);
    println!("{} requests / second", requests as f64 / seconds);

    let mut flat_facts: Vec<Fact> = Vec::new();
    facts.into_iter().for_each(|facts| flat_facts.extend(facts));

    println!("{:?}", Summary::from_facts(&flat_facts));
}
