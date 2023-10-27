use std::thread;

use crate::Result;
use crate::ThreadPool;
use crossbeam::channel::{self, Receiver, Sender};

pub struct SharedQueueThreadPool {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let (tx, rx) = channel::unbounded();
        for _ in 0..threads {
            let rx = rx.clone();
            thread::spawn(move || TaskReceiver { rx }.run());
        }

        Ok(Self { tx })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Box::new(job)).unwrap();
    }
}

struct TaskReceiver {
    rx: Receiver<Box<dyn FnOnce() + Send + 'static>>,
}

impl TaskReceiver {
    fn run(&mut self) {
        while let Ok(job) = self.rx.recv() {
            job();
        }
    }
}

impl Drop for TaskReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.rx.clone();
            thread::spawn(move || TaskReceiver { rx }.run());
        }
    }
}
