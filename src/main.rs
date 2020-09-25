use std::cell::UnsafeCell;
use std::sync::{Arc, Barrier};
use std::thread;

struct UnthreadSafeStruct {
    data: UnsafeCell<usize>,
}

unsafe impl Sync for UnthreadSafeStruct {}

impl UnthreadSafeStruct {
    pub fn new() -> UnthreadSafeStruct {
        UnthreadSafeStruct {
            data: UnsafeCell::new(0),
        }
    }
}

static NITERS: usize = 2000;

fn main() {
    let my_lock = Arc::new(UnthreadSafeStruct::new());
    let barrier = Arc::new(Barrier::new(NITERS));

    let mut children = vec![];

    for _ in 0..NITERS {
        let my_lock = my_lock.clone();
        let barrier = barrier.clone();
        children.push(thread::spawn(move || {
            barrier.wait();
            let mut value = unsafe { *my_lock.data.get() };
            value += 1;
            unsafe { *my_lock.data.get() = value };
        }));
    }

    for child in children {
        let _ = child.join();
    }

    let value = unsafe { *my_lock.data.get() };
    println!("{:?}", value);
}
