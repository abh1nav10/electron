use crate::Queue;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

type Task = Box<dyn FnOnce() + Send + Sync>;

pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    tasks: Arc<Queue<Task>>,
    execution: Arc<AtomicBool>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.execution.store(false, Ordering::Relaxed);
        let mut counter = 0;
        for thread in self.threads.drain(..) {
            if let Err(_) = thread.join() {
                counter += 1;
            }
        }
        println!("{} threads panicked while joining!", counter);
    }
}

impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        ThreadPool {
            threads: Vec::with_capacity(number),
            tasks: Arc::new(Queue::<Task>::new()),
            execution: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn spawn(&mut self) {
        for _ in 0..self.threads.capacity() {
            let queue: Arc<Queue<Task>> = Arc::from(self.tasks.clone());
            let execution: Arc<AtomicBool> = Arc::from(self.execution.clone());
            let thread = thread::spawn(move || {
                loop {
                    if !execution.load(Ordering::Relaxed) {
                        break;
                    }
                    let res = queue.dequeue();
                    if let Ok(func) = res {
                        let _ = std::panic::catch_unwind(|| func());
                    } else {
                        thread::yield_now();
                    }
                }
            });
            self.threads.push(thread);
        }
    }

    pub fn execute_task<T>(&self, task: T)
    where
        T: FnOnce() + Send + Sync + 'static,
    {
        let boxed = Box::new(task);
        self.tasks.enqueue(boxed);
    }
}
