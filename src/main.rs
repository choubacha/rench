extern crate rayon;
extern crate clap;
extern crate reqwest;
use clap::{Arg, App};
use reqwest::StatusCode;
use std::time::{Instant, Duration};

#[derive(Debug)]
struct Fact {
    status: StatusCode,
    duration: Duration,
    content_length: usize,
}

fn make_request(url: &str) -> Fact {
    let start = Instant::now();
    let resp = reqwest::get(url).expect("Failure to even connect is no good");
    let duration = start.elapsed();
    Fact {
        duration,
        status: resp.status(),
        content_length: 0,
    }
}

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(Arg::with_name("URL").required(true))
        .get_matches();
    let url = matches.value_of("URL").expect("URL is required");
    println!("{:?}", make_request(url));
}
