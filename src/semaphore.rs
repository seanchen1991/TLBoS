use std::sync::{Mutex, Condvar};

pub struct Semaphore {
    counter: Mutex<isize>,
    condvar: Condvar,
}

impl Semaphore {
    pub fn new(n: isize) -> Self {
        Semaphore {
            counter: Mutex::new(n),
            condvar: Condvar::new()
        }
    }

    pub fn acquire(&self) {
        let mut count = self.counter.lock().unwrap();

        while *count <= 0 {
            count = self.condvar.wait(count).unwrap();
        }

        *count -= 1;
    }

    pub fn release(&self) {
        let mut count = self.counter.lock().unwrap();
        
        *count += 1;

        self.condvar.notify_one();
    }
}
