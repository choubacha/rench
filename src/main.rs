extern crate rayon;
extern crate clap;
extern crate reqwest;
use clap::{Arg, App};
use reqwest::{Request, Method, Client};
use std::time::{Instant, Duration};
use std::thread;
use std::sync::mpsc::{channel, Sender};

mod stats;
mod chart;
use stats::{Fact, Summary};


fn make_requests(urls: Vec<String>, number_of_requests: usize, sender: Sender<Option<Fact>>) {
    let client = Client::new();

    (0..number_of_requests).for_each(|n| {
        let url = &urls[n % urls.len()];

        let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
        let (resp, duration) = time_it(|| {
            let mut resp = client.execute(request).expect(
                "Failure to even connect is no good",
            );
            let _ = resp.text().expect("Read the body");
            resp
        });
        sender.send(Some(Fact::record(resp, duration))).expect(
            "to send the fact correctly",
        );
    });
    sender.send(None).expect("to send None correctly");
}

fn time_it<F, U>(f: F) -> (U, Duration)
where
    F: FnOnce() -> U,
{
    let start = Instant::now();
    (f(), start.elapsed())
}

fn distribute_work(threads: usize, requests: usize) -> Vec<usize> {
    // Every thread should get even work:
    let base_work = requests / threads;
    let remaining_work = requests % threads;

    (0..threads)
        .map(|thread| {
            // The remainder means that we don't have enough for
            // every thread to get 1. So we just add one until
            // we've used up the entire remainder
            if thread < remaining_work {
                base_work + 1
            } else {
                base_work
            }
        })
        .collect()
}

#[test]
fn it_can_distribute_all_work_as_evenly_as_possible() {
    assert_eq!(distribute_work(3, 1000), vec![334, 333, 333]);
    assert_eq!(distribute_work(2, 1000), vec![500, 500]);
    assert_eq!(
        distribute_work(20, 39),
        vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1]
    );
}

fn main() {
    let matches = App::new("Git Release Names")
        .author("Kevin Choubacha <chewbacha@gmail.com>")
        .arg(Arg::with_name("URL").required(true).multiple(true))
        .arg(Arg::with_name("concurrency").short("c").takes_value(true))
        .arg(Arg::with_name("requests").short("n").takes_value(true))
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

    let (sender, receiver) = channel::<Option<Fact>>();
    let mut facts: Vec<Fact> = Vec::with_capacity(requests);

    let rec_handle = thread::spawn(move || {
        let mut threads_finished = 0;
        while threads_finished < threads {
            if let Some(fact) = receiver.recv().expect("To receive correctly") {
                facts.push(fact);
                if (facts.len() % (requests / 10)) == 0 {
                    println!("{} requests", facts.len());
                }
            } else {
                threads_finished += 1;
            }
        }
        facts
    });

    println!("Beginning requests");
    let handles: Vec<thread::JoinHandle<()>> = distribute_work(threads, requests)
        .into_iter()
        .map(|work| {
            let urls: Vec<String> = urls.clone();
            let tx = sender.clone();
            thread::spawn(move || make_requests(urls, work, tx))
        })
        .collect();

    let ((), duration) = time_it(|| {
        handles.into_iter().for_each(|h| h.join().expect("Sending thread to finish"));
    });
    let facts = rec_handle.join().expect("Receiving thread to finish");
    let seconds = duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1_000_000_000f64);

    println!("Finished!");
    println!("");
    println!("Took {} seconds", seconds);
    println!("{} requests / second", requests as f64 / seconds);
    println!("");
    println!("{}", Summary::from_facts(&facts));
}
