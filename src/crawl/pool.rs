use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

enum Message {
    Crawl(String),
    Terminate,
}

pub trait Processor {
    fn crawl(&self, url: String);
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new<P: Processor + Send + Sync + 'static>(queue: Arc<Mutex<VecDeque<Message>>>, processor: Arc<P>) -> Worker {
        let t: JoinHandle<()> = thread::spawn(move || {
            let mut sleep = false;
            loop {
                if let Ok(ref mut q) = queue.lock() {
                    match q.pop_back() {
                        Some(Message::Crawl(url)) => processor.crawl(url),
                        Some(Message::Terminate) => return,

                        //queue is drained
                        None => sleep = true,
                    }
                }

                if sleep {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });

        return Worker {
            thread: Some(t),
        };
    }
}

pub struct Pool {
    threads: usize,
    queue: Arc<Mutex<VecDeque<Message>>>,
    crawlers: Vec<Worker>,
}

impl Pool {
    pub fn new<P: Processor + Send + Sync + 'static>(threads: usize, processor: Arc<P>) -> Pool {
        let queue: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
        let mut crawlers = Vec::with_capacity(threads);
        for _ in 0..threads {
            crawlers.push(Worker::new(queue.clone(), processor.clone()));
        }

        return Pool {
            threads,
            queue,
            crawlers,
        };
    }

    pub fn submit(&self, url: String) {
        self.queue.lock().unwrap().push_front(Message::Crawl(url));
    }

    pub fn stop(&mut self) {
        for _ in 0..self.threads {
            self.queue.lock().unwrap().push_front(Message::Terminate);
        }

        for i in 0..self.threads {
            self.crawlers[i].thread.take().unwrap().join();
        }
    }

    pub fn len(&self) -> usize {
        return self.queue.lock().unwrap().len();
    }
}
