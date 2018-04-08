extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate reqwest;
extern crate tokio_core;

use clap::{App, Arg};
use std::thread;
use std::cmp;
use std::sync::mpsc::{channel, Receiver, Sender};

mod content_length;
mod stats;
mod chart;
mod engine;
mod bench;
use stats::{Fact, Summary};

fn make_requests(eng: engine::Engine, sender: &Sender<Message<Fact>>) {
    eng.run(|fact| {
        sender
            .send(Message::Body(fact))
            .expect("to send the fact correctly");
    });
    sender.send(Message::EOF).expect("to send None correctly");
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

enum Message<T> {
    Body(T),
    EOF,
}

fn recv_messages<T>(
    rx: &Receiver<Message<T>>,
    expected_message_count: usize,
    sender_count: usize,
) -> Vec<T> {
    let chunk_size = cmp::max(expected_message_count / 10, 1);
    let mut eof_count = 0;
    let mut messages: Vec<T> = Vec::with_capacity(expected_message_count);

    while eof_count < sender_count {
        match rx.recv().expect("To receive correctly") {
            Message::Body(message) => {
                messages.push(message);
                if (messages.len() % (chunk_size)) == 0 {
                    println!("{} requests", messages.len());
                }
            }
            Message::EOF => eof_count += 1,
        }
    }
    messages
}

#[cfg(test)]
mod message_collection_tests {
    use super::*;

    #[test]
    fn it_ends_when_all_nones_are_received() {
        let (tx, rx) = channel::<Message<usize>>();
        let handle = thread::spawn(move || recv_messages(&rx, 0, 4));
        for _ in 0..4 {
            let _ = tx.send(Message::EOF);
        }
        assert_eq!(handle.join().unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn it_collects_all_data_received() {
        let (tx, rx) = channel::<Message<usize>>();
        let handle = thread::spawn(move || recv_messages(&rx, 0, 1));
        for n in 0..5 {
            let _ = tx.send(Message::Body(n as usize));
        }
        let _ = tx.send(Message::EOF);
        assert_eq!(handle.join().unwrap(), vec![0, 1, 2, 3, 4]);
    }
}

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
                .help("The engine to use"),
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

    let engine_kind = matches.value_of("engine").unwrap_or("reqwest");

    let (sender, receiver) = channel::<Message<Fact>>();

    let rec_handle = thread::spawn(move || recv_messages(&receiver, requests, threads));

    let is_head_requests = matches.is_present("head-requests");

    println!("Beginning requests");
    let handles: Vec<thread::JoinHandle<()>> = distribute_work(threads, requests)
        .into_iter()
        .map(|work| {
            let mut eng = match engine_kind {
                "hyper" => engine::Engine::new(urls.clone(), work).with_hyper(),
                "reqwest" | _ => engine::Engine::new(urls.clone(), work),
            };
            if is_head_requests {
                eng = eng.with_method(engine::Method::Head);
            }
            let tx = sender.clone();
            thread::spawn(move || make_requests(eng, &tx))
        })
        .collect();

    let ((), duration) = bench::time_it(|| {
        handles
            .into_iter()
            .for_each(|h| h.join().expect("Sending thread to finish"));
    });
    let facts = rec_handle.join().expect("Receiving thread to finish");
    let seconds =
        duration.as_secs() as f64 + (f64::from(duration.subsec_nanos()) / 1_000_000_000f64);

    println!("Finished!");
    println!();
    println!("Took {} seconds", seconds);
    println!("{} requests / second", requests as f64 / seconds);
    println!();
    println!("{}", Summary::from_facts(&facts));
}
