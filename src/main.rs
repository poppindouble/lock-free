use std::cell::UnsafeCell;
use std::sync::{Arc, Barrier};
use std::thread;

struct SharedMemory {
    pub num: UnsafeCell<usize>,
}

unsafe impl Sync for SharedMemory {}

fn main() {
    let barrier = Arc::new(Barrier::new(3));
    let shared_mem_x = Arc::new(SharedMemory{num: UnsafeCell::new(0)});
    let shared_mem_y = Arc::new(SharedMemory{num: UnsafeCell::new(0)});
    let shared_mem_r1 = Arc::new(SharedMemory{num: UnsafeCell::new(0)});
    let shared_mem_r2 = Arc::new(SharedMemory{num: UnsafeCell::new(0)});

    for i in 0..10000000 {

        {
            unsafe  {
                *shared_mem_x.num.get() = 0;
                *shared_mem_y.num.get() = 0;
                *shared_mem_r1.num.get() = 0;
                *shared_mem_r2.num.get() = 0;
            }
        }

        let b1 = barrier.clone();
        let smx_1 = shared_mem_x.clone();
        let smy_1 = shared_mem_y.clone();
        let smr1 = shared_mem_r1.clone();
        let t1 = thread::spawn(move || {
            b1.wait();

            unsafe {
                *smx_1.num.get() = 1;
            }

            unsafe {
                *smr1.num.get() = *smy_1.num.get();
            }
        });

        let b2 = barrier.clone();
        let smx_2 = shared_mem_x.clone();
        let smy_2 = shared_mem_y.clone();
        let smr2 = shared_mem_r2.clone();
        let t2 = thread::spawn(move || {
            b2.wait();

            unsafe {
                *smy_2.num.get() = 1;
            }

            unsafe {
                *smr2.num.get() = *smx_2.num.get();
            }
        });

        barrier.wait();
        t1.join().unwrap();
        t2.join().unwrap();

        let r1 = shared_mem_r1.num.get();
        let r2 = shared_mem_r2.num.get();

        unsafe {
            if *r1 == *r2 && *r1 == 0 {
                println!("got it! r1: {:?}, r2: {:?}, current iteration number: {:?}", *r1, *r2, i);
            }
        }
    }
}