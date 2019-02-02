extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate reqwest;
extern crate tokio_core;

use clap::{App, Arg};

mod bench;
mod chart;
mod collector;
mod content_length;
mod engine;
mod message;
mod plan;
mod runner;
mod stats;
use stats::{ChartSize, Fact, Summary};
use plan::Plan;
use runner::Runner;

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(
            Arg::with_name("URL")
                .required(true)
                .multiple(true)
                .help("Each url specified will be round robined."),
        )
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .takes_value(true)
                .help("The number of concurrent requests to make"),
        )
        .arg(
            Arg::with_name("requests")
                .short("n")
                .takes_value(true)
                .help("The number of requests in total to make"),
        )
        .arg(
            Arg::with_name("head-requests")
                .short("i")
                .long("head")
                .help("The issue head requests instead of get"),
        )
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .short("e")
                .takes_value(true)
                .possible_values(&["hyper", "reqwest"])
                .default_value("hyper")
                .help("The engine to use"),
        )
        .arg(
            Arg::with_name("header")
                .long("header")
                .multiple(true)
                .takes_value(true)
                .number_of_values(1)
                .help("Headers to inject in the request. Example '--header user-agent=rust-rench'"),
        )
        .arg(
            Arg::with_name("chart-size")
                .long("chart-size")
                .takes_value(true)
                .possible_values(&["none", "n", "small", "s", "medium", "m", "large", "l"])
                .help("The size of the chart to render"),
        )
        .get_matches();

    let urls: Vec<String> = matches
        .values_of("URL")
        .expect("URLs are required")
        .map(|v| v.to_string())
        .collect();

    let threads = matches
        .value_of("concurrency")
        .unwrap_or("1")
        .parse::<usize>()
        .expect("Expected valid number for threads");

    let requests = matches
        .value_of("requests")
        .unwrap_or("1000")
        .parse::<usize>()
        .expect("Expected valid number for number of requests");

    let chart_size = match matches.value_of("chart-size").unwrap_or("medium") {
        "none" | "n" => ChartSize::None,
        "small" | "s" => ChartSize::Small,
        "medium" | "m" => ChartSize::Medium,
        "large" | "l" => ChartSize::Large,
        _ => unreachable!(),
    };

    let headers: Vec<String> = matches
        .values_of("header")
        .unwrap_or(Default::default())
        .map(|v| v.to_string())
        .collect();

    let plan = Plan::new(threads, requests);

    let eng = match matches.value_of("engine").unwrap_or("hyper") {
        "hyper" => engine::Engine::new(urls.clone(), headers).with_hyper(),
        "reqwest" | _ => engine::Engine::new(urls.clone(), headers),
    };

    let eng = if matches.is_present("head-requests") {
        eng.with_method(engine::Method::Head)
    } else {
        eng
    };

    let (collector, rec_handle) = collector::start::<Fact>(plan);
    let runner = Runner::start(plan, &eng, &collector);

    println!("Beginning requests");
    let ((), duration) = bench::time_it(|| runner.join());
    let facts = rec_handle.join().expect("Receiving thread to finish");
    let seconds =
        duration.as_secs() as f64 + (f64::from(duration.subsec_nanos()) / 1_000_000_000f64);

    println!("Finished!");
    println!();
    println!("Took {} seconds", seconds);
    println!("{} requests / second", requests as f64 / seconds);
    println!();
    println!(
        "{}",
        Summary::from_facts(&facts).with_chart_size(chart_size)
    );
}
