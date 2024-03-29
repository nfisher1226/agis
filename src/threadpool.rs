use std::{
    num::NonZeroUsize,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::log::{Log, LogError};

/// A pool of worker threads to handle requests
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

/// A type alias representing a job for the threadpool
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers");
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    #[must_use]
    /// Starts up the thread pool
    pub fn new(size: NonZeroUsize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(usize::from(size));
        for id in 0..usize::from(size) {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Self { workers, sender }
    }

    /// Passes a job off to the worker
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(e) = self.sender.send(Message::NewJob(job)) {
            if let Err(e) = e.log_err() {
                eprintln!("{e}");
            }
        }
    }

    /// Shuts down the threadpool when finished
    /// # Panics
    /// The threads will panic rather than shut down cleanly if message passing
    /// fails
    pub fn shutdown(&mut self) {
        let _msg = "Sending terminate message to all workers".to_string().log();
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        let _msg = "Shutting down all workers".to_string().log();
        for worker in &mut self.workers {
            let _msg = format!("Dropping worker {}", worker.id).log();
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// A worker thread
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread for the pool
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // Using `try_lock` here causes misbehavior, at least with musl libc,
            // as the call **thinks** it would block. This can fill the error log
            // quickly with spam
            match receiver.lock().map(|x| x.recv()) {
                Err(e) => {
                    if let Err(e) = e.log_err() {
                        eprintln!("{e}");
                    }
                }
                Ok(Err(e)) => {
                    if let Err(e) = e.log_err() {
                        eprintln!("{e}");
                    }
                }
                Ok(Ok(message)) => match message {
                    Message::NewJob(job) => job(),
                    Message::Terminate => {
                        if let Err(e) = format!("Worker {id} shutting down").log() {
                            eprintln!("{e}");
                        }
                        break;
                    }
                },
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}
