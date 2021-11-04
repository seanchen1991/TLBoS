use std::ops::Drop;
use std::sync::{Condvar, Mutex};

/// A counting, blocking, semaphore.
///
/// Semaphores are a form of atomic counter where access is only granted if the counter is a
/// positive value. Each acquisition blocks the calling thread until the counter is positive. Each
/// release increments the counter and unblocks any threads if necessary.
pub struct Semaphore {
    /// The counter, wrapped in a Mutex to ensure atomicity.
    counter: Mutex<isize>,
    /// The condvar notifies any threads that are blocked waiting on the semaphore.
    condvar: Condvar,
}

/// An RAII guard which will release a resource acquired from a semaphore when dropped.
pub struct SemaphoreGuard<'a> {
    /// The semaphore being guarded.
    sem: &'a Semaphore,
}

impl Semaphore {
    /// Initialize a new semaphore with the initial count specified.
    ///
    /// The count can be thought of as the number of resources that the semaphore is protecting.
    /// A call to `acquire` or `access` will block until at least one resource is available. It is
    /// valid to initialize a semaphore with a negative count.
    pub fn new(n: isize) -> Self {
        Semaphore {
            counter: Mutex::new(n),
            condvar: Condvar::new(),
        }
    }

    /// Acquires the resource protected by the semaphore, blocking the current thread until the
    /// resource is actually acquired.
    ///
    /// If no resources are available, the thread will be blocked waiting on the resource until one
    /// is available.
    pub fn acquire(&self) {
        let mut count = self.counter.lock().unwrap();
        while *count <= 0 {
            count = self.condvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    /// Release a resource from the semaphore.
    ///
    /// Increments the semaphore's count and notifies any pending threads if necssary.
    pub fn release(&self) {
        let mut count = self.counter.lock().unwrap();
        *count += 1;
        self.condvar.notify_one();
    }

    /// Acquires a resource of this semaphore, returning an RAII guard to release the semaphore
    /// when the guard is dropped.
    ///
    /// This function is semantically equivalent to an `acquire` followed by a `release` when the
    /// returned guard is dropped.
    pub fn access(&self) -> SemaphoreGuard {
        self.acquire();
        SemaphoreGuard { sem: self }
    }
}

// Implement the Drop trait to specify that the SemaphoreGuard should release the semaphore when
// the guard goes out of scope.
impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        self.sem.release()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_sem_acquire_release() {
        let sem = Semaphore::new(1);
        sem.acquire();
        sem.release();
        sem.acquire();
    }

    #[test]
    fn test_sem_basic() {
        let s = Semaphore::new(1);
        let _ = s.access();
    }

    #[test]
    fn test_sem_as_mutex() {
        let s = Arc::new(Semaphore::new(1));
        let s2 = s.clone();

        let _t = thread::spawn(move || {
            let _g = s2.access();
        });

        let _g = s.access();
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

    #[test]
    fn test_sem_multi_resource() {
        let s = Arc::new(Semaphore::new(2));
        let s2 = s.clone();

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let _t = thread::spawn(move || {
            let _g = s2.access();
            let _ = rx2.recv();
            tx1.send(()).unwrap();
        });

        let _g = s.access();
        tx2.send(()).unwrap();
        rx1.recv().unwrap();
    }

    #[test]
    fn test_sem_runtime_friendly_blocking() {
        let s = Arc::new(Semaphore::new(2));
        let s2 = s.clone();
        let (tx, rx) = channel();

        {
            let _g = s.access();
            thread::spawn(move || {
                tx.send(()).unwrap();
                drop(s2.access());
                tx.send(()).unwrap();
            });

            rx.recv().unwrap();
        }

        rx.recv().unwrap();
    }
}
