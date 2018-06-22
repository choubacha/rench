use message::Message;
use plan::Plan;
use std::{
    cmp, sync::mpsc::{channel, Receiver, Sender}, thread,
};

/// Kicks off the collector which is a background thread. The collector will capture
/// all data sent to the sender and then will return on the handle the entire dataset.
///
/// The plan is essential to pre-allocating the array.
pub fn start<T>(plan: Plan) -> (Sender<Message<T>>, thread::JoinHandle<Vec<T>>)
where
    T: 'static + Send,
{
    let (sender, receiver) = channel::<Message<T>>();
    (sender, thread::spawn(move || collect(&receiver, plan)))
}

fn collect<T>(receiver: &Receiver<Message<T>>, plan: Plan) -> Vec<T>
where
    T: 'static + Send,
{
    let chunk_size = cmp::max(plan.requests() / 10, 1);
    let mut eof_count = 0;
    let mut messages: Vec<T> = Vec::with_capacity(plan.requests());

    while eof_count < plan.threads() {
        match receiver.recv().expect("To receive correctly") {
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
        let plan = Plan::new(4, 0);
        let (tx, handle) = start::<usize>(plan);
        for _ in 0..4 {
            let _ = tx.send(Message::EOF);
        }
        assert_eq!(handle.join().unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn it_collects_all_data_received() {
        let plan = Plan::new(1, 0);
        let (tx, handle) = start::<usize>(plan);
        for n in 0..5 {
            let _ = tx.send(Message::Body(n as usize));
        }
        let _ = tx.send(Message::EOF);
        assert_eq!(handle.join().unwrap(), vec![0, 1, 2, 3, 4]);
    }
}
