use std::thread;

use super::ThreadPool;
use crate::Result;

pub struct NaiveThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool for NaiveThreadPool {
    fn new(size: u32) -> Result<Self> {
        assert!(size > 0);
        let mut threads = Vec::with_capacity(size as usize);
        for _ in 0..size {
            threads.push(thread::spawn(|| {}));
        }
        Ok(Self { threads })
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(f);
    }
}
