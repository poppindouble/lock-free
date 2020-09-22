use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::{thread, time};

#[derive(Clone)]
struct LazyTransformer<FN: Fn(u16) -> bool> {
    pub source: Arc<AtomicU16>,
    pub value: Arc<AtomicBool>,
    pub transform_fn: FN,
}

impl<FN: Fn(u16) -> bool> LazyTransformer<FN> {
    pub fn new(transform_fn: FN) -> Self {
        LazyTransformer {
            source: Arc::new(AtomicU16::new(0)),
            value: Arc::new(AtomicBool::new(true)),
            transform_fn,
        }
    }

    pub fn set_source(&self, source: u16) {
        (*self.source).store(source, Ordering::Release);
    }

    pub fn get_transformed(&self) -> bool {
        let pre_source = self.source.swap(0, Ordering::AcqRel);
        if pre_source != 0 {
            let new_value = (self.transform_fn)(pre_source);
            self.value.store(new_value, Ordering::Release);
            return new_value;
        } else {
            let cached_value = self.value.load(Ordering::Acquire);
            return cached_value;
        }
    }
}

fn main() {
    let transform_fn = Box::new(|hold_val| {
        let sec = time::Duration::from_secs(5);
        println!("executing transform for {:?}.", sec);
        thread::sleep(sec);
        return hold_val % 2 == 0;
    });
    let lazy_transformer = LazyTransformer::new(transform_fn);
    let mut handles = vec![];

    for i in 0..1000 {
        let lazy_clone = lazy_transformer.clone();
        let handle = thread::spawn(move || {
            let sec = time::Duration::from_millis(100 * i);
            thread::sleep(sec);
            let value = lazy_clone.get_transformed();
            println!("getting value {:?}", value);
        });
        handles.push(handle);
    }

    println!("launched all readers");

    for i in 0..10 {
        let lazy_clone = lazy_transformer.clone();
        let handle = thread::spawn(move || {
            let sec = time::Duration::from_secs(i);
            thread::sleep(sec);
            println!("setting source {:?}", i);
            lazy_clone.set_source((i + 1) as u16);
        });
        handles.push(handle);
    }

    println!("launched all setters");

    for handle in handles {
        handle.join().unwrap();
    }
}
