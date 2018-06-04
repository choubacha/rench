use crossbeam;
use crossbeam_channel::{self as channel, Receiver, Sender};
use std::thread;

/// Starts a collector and feeds the client to the caller. This wraps the entire
/// implementation of the server itself so that the caller doesn't need to
/// know how to start it.
///
/// Clone the client to share between threads.
pub fn with<T>(mut f: impl FnMut(Client<T>)) -> Vec<T>
where
    T: Send + Clone,
{
    crossbeam::scope(|scope| {
        let mut server = Server::new();
        let client = server.build_client();

        let handle = scope.spawn(move || {
            server.listen();
            server.data
        });

        f(client.clone());

        client.stop();
        handle.join()
    })
}

/// A client to the server. This will allow other services to
/// send data back to the collector. This is a cloneable value
/// and can be cloned to give to other threads.
#[derive(Debug, Clone)]
pub struct Client<T>
where
    T: Send + Clone,
{
    sender: Sender<Message<T>>,
}

impl<T> Client<T>
where
    T: Send + Clone,
{
    /// Send a value to the collector.
    pub fn send(&self, message: T) {
        self.sender.send(Message::Body(message));
    }

    /// Sends the message to halt the server. It only takes one to kill it.
    fn stop(&self) {
        self.sender.send(Message::Stop);
    }
}

#[derive(Debug)]
enum Message<T>
where
    T: Send + Clone,
{
    Body(T),
    Stop,
}

/// Represents the collector service. This value provides the ability
/// to collect the results into a single vector.
#[derive(Debug)]
struct Server<T>
where
    T: Send + Clone,
{
    receiver: Receiver<Message<T>>,
    sender: Sender<Message<T>>,
    data: Vec<T>,
}

impl<T> Server<T>
where
    T: Send + Clone,
{
    /// Create a new server for collecting.
    ///
    /// This server can be backgrounded by moving to a thread. If moving to a thread
    /// you should return the data value after listening ends so that it can be returned.
    ///
    /// This generally would require the type to be static but using crossbeam's scoped
    /// threads we can do it on the stack without needing static.
    fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        Self {
            sender,
            receiver,
            data: Vec::new(),
        }
    }

    /// Returns a client for the current implementation that can be used to send
    /// data back to collector.
    fn build_client(&self) -> Client<T> {
        Client {
            sender: self.sender.clone(),
        }
    }

    /// Starts popping off the queue and appending data in the data array.
    fn listen(&mut self) {
        while let Some(Message::Body(item)) = self.receiver.recv() {
            self.data.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_collect() {
        let server = Server::<()>::new();
        assert_eq!(server.data, Vec::new());
    }

    #[test]
    fn send_and_receive() {
        let mut server = Server::<u32>::new();
        let client = server.build_client();
        client.send(123);
        client.stop();
        server.listen();
        assert_eq!(server.data, vec![123]);
    }

    #[test]
    fn with_collector() {
        let results = with(|client| {
            client.send(123);
            let h = thread::spawn(move || client.send(456));
            h.join().unwrap();
        });

        assert_eq!(results, vec![123, 456]);
    }
}
