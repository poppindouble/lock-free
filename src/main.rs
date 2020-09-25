use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

struct MyLock {
    flag: Arc<AtomicBool>,
    data: UnsafeCell<usize>,
}

unsafe impl Sync for MyLock {}

impl MyLock {
    pub fn new() -> MyLock {
        MyLock {
            flag: Arc::new(AtomicBool::new(false)),
            data: UnsafeCell::new(0),
        }
    }

    pub fn try_lock(&self) -> Option<MyLockGuard> {
        let was_locked = self.flag.swap(true, Ordering::Acquire);
        if was_locked {
            None
        } else {
            Some(MyLockGuard { guard: self })
        }
    }
}

struct MyLockGuard<'a> {
    guard: &'a MyLock,
}

impl<'a> Drop for MyLockGuard<'a> {
    fn drop(&mut self) {
        self.guard.flag.store(false, Ordering::Release);
    }
}

static NITERS: usize = 2000;

fn main() {
    let my_lock = Arc::new(MyLock::new());
    let barrier = Arc::new(Barrier::new(NITERS));

    let mut children = vec![];

    for _ in 0..NITERS {
        let my_lock = my_lock.clone();
        let barrier = barrier.clone();
        children.push(thread::spawn(move || {
            barrier.wait();
            loop {
                if let Some(guard) = my_lock.try_lock() {
                    // single thread accessing here.

                    let mut value = unsafe { *guard.guard.data.get() };
                    value += 1;
                    unsafe { *guard.guard.data.get() = value };
                    break;
                } else {
                    // fail to unlock
                }
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }

    let value = unsafe { *my_lock.data.get() };
    println!("{:?}", value);
}
