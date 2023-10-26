use crate::Result;
use crate::ThreadPool;

pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NaiveThreadPool {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = std::thread::spawn(job);
    }
}
