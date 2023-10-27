use crate::Result;

pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub mod naive;
pub mod rayon;
pub mod sharedqueue;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use sharedqueue::SharedQueueThreadPool;
