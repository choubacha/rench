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

fn make_request(url: &str) {
    let client = Client::new();

    // Warm up
    let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
    let resp = client.execute(request).expect("Failure to even connect is no good");

    for _ in 0..100 {
        let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
        let start = Instant::now();
        let resp = client.execute(request).expect("Failure to even connect is no good");
        let duration = start.elapsed();
        let fact = Fact { duration, status: resp.status(), content_length: 0, };
        println!("{:?}", fact);
    }
}

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(Arg::with_name("URL").required(true))
        .get_matches();
    let url = matches.value_of("URL").expect("URL is required");
    make_request(url);
}
