use engine::Engine;
use plan::Plan;
use message::Message;
use stats::Fact;
use std::{thread, sync::mpsc::Sender};

/// The runner struct represents an ongoing run time of the engine.
pub struct Runner {
    handles: Vec<thread::JoinHandle<()>>,
}

impl Runner {
    /// Launches the runner with a plan. It will tell the engine to run and broadcast the
    /// facts that the engine produces. The plan tells the runner how many threads to run
    /// on and how to distribute the work.
    pub fn start(plan: Plan, eng: &Engine, collector: &Sender<Message<Fact>>) -> Runner {
        let handles = plan.distribute()
            .into_iter()
            .map(|work| {
                let collector = collector.clone();
                let eng = eng.clone();
                thread::spawn(move || Self::run(work, eng, &collector))
            })
            .collect();
        Runner { handles }
    }

    /// After the runner has been started, it just be joined so that all of the work can
    /// be finished.
    pub fn join(self) {
        self.handles
            .into_iter()
            .for_each(|h| h.join().expect("Sending thread to finish"));
    }

    fn run(work: usize, eng: Engine, collector: &Sender<Message<Fact>>) {
        eng.run(work, |fact| {
            collector
                .send(Message::Body(fact))
                .expect("to send the fact correctly");
        });
        collector
            .send(Message::EOF)
            .expect("to send None correctly");
    }
}
