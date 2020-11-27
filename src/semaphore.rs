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

#[cfg(test)]
mod tests {
    use super::Semaphore;
    use std::thread;
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    #[test]
    fn test_sem_acquire_release() {
        let sem = Semaphore::new(1);

        sem.acquire();
        sem.release();
        sem.acquire();
    }

    #[test]
    fn test_child_waits_parent_signals() {
        let s1 = Arc::new(Semaphore::new(0));
        let s2 = s1.clone();
        
        let (tx, rx) = channel();
         
        let _t = thread::spawn(move || {
            s2.acquire();
            tx.send(()).unwrap();
        });

        s1.release();
        let _ = rx.recv();
    }
    
    #[test]
    fn test_parent_waits_child_signals() {
        let s1 = Arc::new(Semaphore::new(0));
        let s2 = s1.clone();

        let (tx, rx) = channel();

        let _t = thread::spawn(move || {
            s2.release();
            let _ = rx.recv();
        });

        s1.acquire();
        tx.send(()).unwrap();
    }
}
